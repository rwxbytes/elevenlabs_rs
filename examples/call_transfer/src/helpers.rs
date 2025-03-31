use crate::{
    AppState, CallTransferData, LABEL_WAIT_MANAGER, PATH_AMD, PATH_EVENTS_CALL, PATH_WS_CALLER,
};
use elevenlabs_twilio::{
    AgentWebSocket, CallStatus, ClientToolResult, CreateCall, CreateCallBody, CreateParticipant,
    CreateParticipantBody, Stream, VoiceResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn, Instrument, Span};

pub(crate) async fn send_tool_error(
    agent_ws: &Arc<Mutex<AgentWebSocket>>,
    tool_id: &str,
    message: &str,
) {
    let error_result = ClientToolResult::new(tool_id)
        .is_error(true)
        .with_result(message.to_string());
    if let Err(e) = agent_ws.lock().await.send_tool_result(error_result).await {
        error!("Error sending tool error result: {:?}", e);
    }
}

#[instrument(name = "call_transfer_retry", skip(state))]
pub(crate) async fn handle_call_transfer_retry(
    state: &AppState,
    call_sid: &str,
    call_status: CallStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    //info!(status = ?call_status, "Handling call transfer retry logic");
    info!("Handling call transfer retry logic");
    let twilio_c = state.telephony_state.twilio_client.clone();

    let (mut new_call_sid_opt, mut updated_state_opt) = (None, None::<CallTransferData>);
    let mut retry_state = state.call_transfer_state.lock().await;
    let needs_update = if let Some(current_state) = retry_state.get_mut(call_sid) {
        // Add context for the specific call being processed
        Span::current().record("conf.name", current_state.conference_name.as_str());
        Span::current().record("retry.count", current_state.retry_count);
        Span::current().record("retry.max", current_state.max_retries);
        info!("Found existing retry state for call");

        match call_status {
            CallStatus::NoAnswer | CallStatus::Busy | CallStatus::Failed => {
                current_state.retry_count += 1;
                Span::current().record("retry.count", current_state.retry_count);

                if current_state.retry_count <= current_state.max_retries {
                    info!(
                        retry.attempt = current_state.retry_count,
                        "Attempting retry"
                    );

                    // --- Perform Retry Call using .instrument() ---
                    let ws_url = format!("{}{}", state.config.wss_ngrok_base_url, PATH_WS_CALLER);
                    let retry_call_fut = async {
                        let twiml = VoiceResponse::new()
                            .connect(Stream::new(ws_url))
                            .to_string()?;

                        let status_callback_url =
                            format!("{}{}", state.config.https_ngrok_base_url, PATH_EVENTS_CALL);
                        let amd_url = format!("{}{}", state.config.https_ngrok_base_url, PATH_AMD);
                        let body = CreateCallBody {
                            to: &state.config.participant_b_phone_number,
                            from: &state.config.warm_transfer_agent_phone_number,
                            twiml: Some(&twiml),
                            timeout: Some(state.config.participant_b_call_timeout_secs),
                            status_callback: Some(&status_callback_url),
                            machine_detection: Some("Enable"),
                            async_amd: Some(true),
                            async_amd_status_callback: Some(amd_url.as_str()),
                            ..Default::default()
                        };
                        debug!(?body, "Prepared retry call creation body");
                        twilio_c
                            .hit(CreateCall::new(twilio_c.account_sid(), body))
                            .await
                    };

                    let retry_call_span = tracing::info_span!("create_retry_call");
                    match retry_call_fut.instrument(retry_call_span).await {
                        Ok(new_call) => {
                            let new_call_sid = new_call.sid.clone();
                            info!(new_call.sid = %new_call_sid, "Retry call initiated successfully");
                            new_call_sid_opt = Some(new_call_sid);
                            updated_state_opt = Some(current_state.clone());
                        }
                        Err(e) => {
                            error!(error = ?e, "Failed to create retry call");
                            // Handle...
                        }
                    }

                    true // Needs update outside lock, as multiple &mut
                } else {
                    // Add an agent to the conference to notify the caller that participant B is unavailable
                    warn!("Max retries reached. Notifying caller if possible.");
                    if let Some(caller_sid) = current_state.caller_call_sid.clone() {
                        let notify_fut = async {
                            let body = CreateParticipantBody {
                                from: &state.config.warm_transfer_agent_phone_number,
                                to: &state.config.wait_management_agent_phone_number,
                                // TODO: When Wait Manager exits because caller wants to wait a bit longer
                                // have a fn to put caller on hold so the hold music starts to play
                                end_conference_on_exit: Some(false),
                                beep: Some(false),
                                conference_status_callback_event: vec!["join"],
                                label: Some(LABEL_WAIT_MANAGER),
                                ..Default::default()
                            };

                            let endpoint = CreateParticipant::new(
                                twilio_c.account_sid(),
                                &current_state.conference_name,
                                body,
                            );
                            twilio_c.hit(endpoint).await
                        };
                        let notify_span = tracing::info_span!("notify_caller_unavailability", caller.sid = %caller_sid);

                        match notify_fut.instrument(notify_span).await {
                            Ok(_) => info!("Successfully initiated notification to caller"),
                            Err(e) => {
                                error!(error = ?e, "Failed to initiate notification to caller")
                            }
                        }
                    } else {
                        warn!("Cannot notify caller: Caller CallSid not found in state.");
                    }
                    // Remove state after max retries reached
                    let data = retry_state.remove(call_sid);
                    info!("Removed retry state after max retries.");
                    let mut guard = state.to_be_notified.lock().await;
                    // Add the data to the queue for the Wait Manager
                    guard.push_back(data.unwrap());
                    info!("Added data to the queue for the Wait Manager");
                    false // No update needed outside lock
                }
            }
            CallStatus::Completed | CallStatus::Canceled => {
                info!("Call finished. Removing retry state.");
                retry_state.remove(call_sid);
                false // No update needed outside lock
            }
            _ => {
                debug!("No retry action needed for this status.");
                false
            }
        }
    } else {
        warn!(
            "Received status update for a call SID not tracked for retries (or already removed). Ignoring."
        );
        false
    };
    drop(retry_state);

    // Update the state map after releasing the lock on the specific entry
    if needs_update && new_call_sid_opt.is_some() {
        if let (Some(new_call_sid), Some(updated_state)) = (new_call_sid_opt, updated_state_opt) {
            let mut retry_state_global = state.call_transfer_state.lock().await;
            retry_state_global.remove(call_sid); // Remove old entry
            retry_state_global.insert(new_call_sid.clone(), updated_state); // Add new entry
            info!(new_call.sid = %new_call_sid, old_call.sid = %call_sid, "Updated call transfer state map for retry");
        }
    }
    Ok(())
}
