use crate::{
    helpers::handle_call_transfer_retry, AppState, CallTransferData, CustomerConversation, LABEL_CALLER, LABEL_WAIT_MANAGER,
    PATH_AMD, PATH_EVENTS_CALL, PATH_WAIT_MANAGER, PATH_WS_CALLER,
    PATH_WS_PARTICIPANT_B, WARM_TRANSFER_AGENT,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Form;
use elevenlabs_twilio::CallStatus::Ringing;
use elevenlabs_twilio::{
    AMDRequestParams, AnsweredBy, CallStatus, ConferenceEvent, ConferenceRequestParams,
    ConversationInitiationClientData, CreateCallBody, FetchParticipant, InboundCall, PostCall,
    Stream, TwilioClient, TwilioParams, TwilioRequestParams, UpdateCall, UpdateCallBody,
    UpdateCallStatus, VoiceResponse,
};
use std::collections::HashMap;
use tracing::{debug, error, field, info, instrument, warn, Instrument, Span};

#[instrument(
    name = "inbound_call",
    skip(state, inbound_call),
    fields(
        call_sid = %inbound_call.params.call_sid,
        //from = %inbound_call.params.from, // Uncomment if needed
        //to = %inbound_call.params.to, // Uncomment if needed
    )
)]
pub(crate) async fn inbound_call_handler(
    State(state): State<AppState>,
    inbound_call: InboundCall,
) -> impl IntoResponse {
    info!("Received inbound call request");
    if inbound_call.params.from == state.config.warm_transfer_agent_phone_number {
        let ws_url = format!("{}{}", &state.config.wss_ngrok_base_url, PATH_WAIT_MANAGER);
        let mut guard = state.to_be_notified.lock().await;
        let data = guard.pop_front().unwrap(); // TODO: Handle empty queue case

        let f = async move || data.to_convo_initiation_client_data();

        inbound_call.answer_and_config(ws_url, f).map_err(|e| {
            error!(error = ?e, "Failed to answer call");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
    } else {
        let ws_url = format!("{}{}", &state.config.wss_ngrok_base_url, PATH_WS_CALLER);
        let caller_num = state.config.caller_phone_number.clone();
        let pred = move |p: &TwilioRequestParams| p.from == caller_num;
        inbound_call.answer_if(ws_url, pred).map_err(|e| {
            error!(error = ?e, "Failed to answer call");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
    }
}

#[instrument(
    name = "conference_event",
    skip_all, // Skip Form params by default, add specific fields
    fields(
        conf.name = %params.friendly_name,
        conf.sid = %params.conference_sid,
        event.type = ?params.status_callback_event,
        event.call_sid = params.call_sid.as_deref().unwrap_or("N/A"),
        sequence_number = %params.sequence_number,
        //timestamp = %params.timestamp,
    )
)]
pub(crate) async fn conference_events_handler(
    State(state): State<AppState>,
    Form(params): Form<ConferenceRequestParams>,
) -> impl IntoResponse {
    let twilio_c = state.telephony_state.twilio_client.clone();
    let mut conference_guard = state.conference_state.lock().await;

    if let Some(ref event) = params.status_callback_event {
        match event {
            ConferenceEvent::ParticipantJoin => {
                let call_sid = params.call_sid.as_deref().unwrap_or("N/A");

                let fetch_participant_fut = async {
                    let fetch_participant = FetchParticipant::new(
                        twilio_c.account_sid(),
                        &params.conference_sid,
                        call_sid,
                    );
                    twilio_c.hit(fetch_participant).await
                };

                let fetch_span = tracing::info_span!("fetch_participant_details");
                match fetch_participant_fut.instrument(fetch_span).await {
                    Ok(resp) => {
                        let label = resp.label.as_deref().unwrap_or("N/A");
                        info!(participant.label = label, "Fetched participant details");
                        conference_guard
                            .update_conference_params(&params.friendly_name, params.clone())
                            .await;
                    }
                    Err(e) => {
                        error!(error = ?e, "Failed to fetch participant details");
                    }
                }
            }
            ConferenceEvent::ParticipantLeave => {
                //TODO: Wait Manager will exit and enter without ending conf in an elaborated example
                let call_sid = params.call_sid.as_deref().unwrap_or("N/A");
                if call_sid != "N/A" {
                    info!("Participant left conference");
                    conference_guard
                        .remove_conference(&params.friendly_name)
                        .await;
                } else {
                    warn!("Participant left but CallSid is missing in event");
                }
            }
            ConferenceEvent::ConferenceEnd => {
                info!("Conference ended");
                conference_guard
                    .remove_conference(&params.friendly_name)
                    .await;
            }
            ConferenceEvent::ConferenceStart => {
                info!("Conference started");
                conference_guard
                    .update_conference_params(&params.friendly_name, params.clone())
                    .await;
            }
            _ => {
                debug!(event.raw = ?event, "Unhandled conference event type");
            }
        }
    } else {
        warn!("Received conference callback without StatusCallbackEvent");
    }
    (StatusCode::OK, "Webhook received")
}

#[instrument(
    name = "post_call",
    skip(state, post_call),
    fields(
        //agent.id = %post_call.payload.data.agent_id,
        convo.id = %post_call.payload.data.conversation_id,
        conf.name = field::Empty,
    )
)]
pub(crate) async fn post_call_handler(
    State(state): State<AppState>,
    post_call: PostCall,
) -> impl IntoResponse {
    info!("Processing post-call event");

    // webhook override ?
    if post_call.payload.data.agent_id != state.config.assessment_agent_id {
        info!("Ignoring post-call webhook for non-assessment agent");
        return (StatusCode::OK, "Webhook received");
    }

    // Ideally we could get the conference name and caller name from ToolResult
    // but often times the tool will execute while its result will not be in the array.

    // add prefix to conference name
    let conference_friendly_name = format!("Conf_{}", &post_call.payload.data.conversation_id);
    // Add conference name to current span context
    Span::current().record("conf.name", &conference_friendly_name.as_str());

    let summary = post_call.summary().unwrap_or_else(|| {
        warn!("No summary found in post-call data");
        "No summary available."
    });

    info!("Initiating call transfer process");

    let customer_conversation = CustomerConversation {
        //caller_name: get for dynamic first msg ?
        summary: summary.to_string(),
        conference_friendly_name: conference_friendly_name.clone(),
    };

    let init_data: ConversationInitiationClientData = customer_conversation.clone().into();

    {
        let mut agent_registry_guard = post_call.agent_registry.lock().await;
        if let Some(agent_arc) = agent_registry_guard.get_mut(WARM_TRANSFER_AGENT) {
            let mut agent_guard = agent_arc.lock().await;
            agent_guard.with_conversation_initiation_client_data(init_data);
            info!(
                agent.target = WARM_TRANSFER_AGENT,
                "Set initial data for target agent"
            );
        } else {
            error!(
                agent.target = WARM_TRANSFER_AGENT,
                "Target agent not found in registry"
            );
            return (StatusCode::OK, "Webhook received");
        }
    }

    // --- Initiate the first call to Participant B ---

    let create_call_fut = async {
        let participant_b_ws_url = format!(
            "{}{}",
            &state.config.wss_ngrok_base_url, PATH_WS_PARTICIPANT_B
        );
        let amd_url = format!("{}{}", &state.config.https_ngrok_base_url, PATH_AMD);
        let stream_noun = Stream::new(&participant_b_ws_url);

        let twiml = VoiceResponse::new().connect(stream_noun).to_string()?;

        let status_callback_url =
            format!("{}{}", &state.config.https_ngrok_base_url, PATH_EVENTS_CALL);
        let create_call_body = CreateCallBody {
            to: &state.config.participant_b_phone_number,
            from: &state.config.warm_transfer_agent_phone_number,
            timeout: Some(state.config.participant_b_call_timeout_secs),
            twiml: Some(&twiml),
            status_callback: Some(&status_callback_url),
            status_callback_event_completed: Some(true),
            status_callback_event_answered: Some(true),
            machine_detection: Some("Enable"),
            async_amd: Some(true),
            async_amd_status_callback: Some(amd_url.as_str()),
            ..Default::default()
        };
        debug!(
            ?create_call_body,
            "Prepared Participant B call creation body"
        );
        post_call.create_call(create_call_body).await
    };

    let create_call_span = tracing::info_span!(
        "create_participant_b_call",
        participant_b.num = %state.config.participant_b_phone_number,
        warm_transfer_agent.call_sid = field::Empty, // Will record on success
        //caller.call_sid = field::Empty, // Will record later
    );

    let caller_call_sid = {
        let guard = state.conference_state.lock().await;
        guard.get_caller_call_sid(&conference_friendly_name).await
    };

    if caller_call_sid.is_none() {
        warn!(
            "Caller SID not found for conference when initiating call to Participant B. Caller might have left."
        );
        return (StatusCode::OK, "Webhook received");
    }

    match create_call_fut.instrument(create_call_span).await {
        Ok(call_instance) => {
            let warm_transfer_agent_call_sid = call_instance.sid.clone();
            Span::current().record(
                "warm_transfer_agent.call_sid",
                &warm_transfer_agent_call_sid.as_str(),
            );
            info!("Successfully initiated call");

            // --- Store Initial Retry State ---
            let initial_retry_state = CallTransferData {
                //caller_name: // TODO: get from post_call
                conference_name: conference_friendly_name.clone(),
                caller_call_sid,
                retry_count: 0,
                max_retries: state.config.max_call_transfer_retries,
                participant_b_phone_number: state.config.participant_b_phone_number.clone(),
                warm_transfer_agent_phone_number: state
                    .config
                    .warm_transfer_agent_phone_number
                    .clone(),
                customer_summary: customer_conversation.summary,
            };

            let mut retry_state_guard = state.call_transfer_state.lock().await;
            retry_state_guard.insert(warm_transfer_agent_call_sid.clone(), initial_retry_state);
            info!("Stored initial call transfer state");

            (StatusCode::OK, "Webhook received")
        }
        Err(e) => {
            error!(error = ?e, "Error creating call to Participant B");
            (StatusCode::OK, "Webhook received")
        }
    }
}

#[instrument(
    name = "participant_b_call_status",
    skip(state, params),
    fields(
        call.sid = %params.call_sid,
        call.status = %params.call_status,
    )
)]
pub(crate) async fn participant_b_callback_handler(
    State(state): State<AppState>,
    params: TwilioParams,
) -> impl IntoResponse {
    info!("Received Participant B call status update");

    let retry_result =
        handle_call_transfer_retry(&state, &params.call_sid, params.call_status.clone())
            .instrument(Span::current()) // Pass context into the function call
            .await;

    if let Err(e) = retry_result {
        error!(error = ?e, "Error occurred during call transfer retry logic");
    }

    (StatusCode::OK, "Webhook received")
}

#[instrument(
    name = "amd_result",
    skip(state, params),
    fields(
       amd.call_sid = %params.call_sid,
       amd.answered_by = ?params.answered_by,
    )
)]
pub(crate) async fn amd_handler(
    State(state): State<AppState>,
    Form(params): Form<AMDRequestParams>,
) -> impl IntoResponse {
    info!("Processing AMD webhook");

    if params.answered_by == AnsweredBy::MachineStart {
        warn!("Answering machine detected. Hanging up call.");

        // --- Hang up call using .instrument() ---
        let hangup_fut = async {
            let twilio_client = match TwilioClient::from_env() {
                Ok(client) => client,
                Err(e) => {
                    error!(error = ?e, "Failed to create Twilio client for AMD hangup");
                    return Err(e);
                }
            };
            let hangup_body = UpdateCallBody {
                status: Some(UpdateCallStatus::Completed),
                ..Default::default()
            };
            let endpoint =
                UpdateCall::new(twilio_client.account_sid(), &params.call_sid, hangup_body);
            twilio_client.hit(endpoint).await
        };

        let hangup_span = tracing::info_span!("hangup_amd_call");
        match hangup_fut.instrument(hangup_span).await {
            Ok(_) => info!("Successfully initiated hangup due to AMD"),
            Err(e) => error!(error = ?e, "Failed to hang up call after AMD detection"),
        }

        let retry_future =
            handle_call_transfer_retry(&state, &params.call_sid, CallStatus::NoAnswer);
        let _ = retry_future.instrument(Span::current()).await; // Instrument with amd_result span
    } else if params.answered_by == AnsweredBy::Human {
        info!("Human answered (AMD result). Call proceeding.");
    } else {
        info!("Unknown or irrelevant AMD result.");
    }

    StatusCode::OK
}
