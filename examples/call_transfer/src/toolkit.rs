use crate::{
    helpers::send_tool_error, AppState, Config, LABEL_CALLER, LABEL_PARTICIPANT_B,
    PATH_EVENTS_CONFERENCE,
};
use elevenlabs_twilio::{
    AgentWebSocket, ClientToolResult, Conference, PhoneCallTool, TwilioClient, TwilioClientExt,
    VoiceResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Debug)]
enum ToolName {
    PutCallerInConference,
    PutHumanOperatorInConference,
    Unknown(String),
}

impl From<&str> for ToolName {
    fn from(name: &str) -> Self {
        match name {
            "put_caller_in_conference" => ToolName::PutCallerInConference,
            "put_human_operator_in_conference" => ToolName::PutHumanOperatorInConference,
            other => ToolName::Unknown(other.to_string()),
        }
    }
}

pub(crate) async fn handle_tool_call(
    tool_call: &PhoneCallTool,
    agent_ws: &Arc<Mutex<AgentWebSocket>>, // remove this perhaps
    twilio_c: &TwilioClient,
    state: &AppState,
) -> Result<(), String> {
    match ToolName::from(tool_call.client_tool_call.name()) {
        ToolName::PutCallerInConference => {
            info!("Tool Execution: PutCallerInConference");
            // add caller name to a state variable
            //let caller_name = tool_call.client_tool_call.parameters().get("caller_name");

            let convo_id = tool_call
                .conversation_id
                .clone()
                .ok_or("Missing conversation ID".to_string())?;

            // use the conversation ID as suffix for the conference name, then we'll strip the prefix on post_call.
            let conf_name = format!("Conf_{}", convo_id);

            let twiml = create_conference_twiml(
                &conf_name,
                LABEL_CALLER,
                false,
                true,
                &state.config.https_ngrok_base_url,
            )?;

            twilio_c
                .update_call_with_twiml(&tool_call.call_sid, &twiml)
                .await
                .map_err(|e| e.to_string())?;

            let mut guard = state.conference_state.lock().await;
            guard
                .add_conference(conf_name.clone(), tool_call.call_sid.clone())
                .await;

            Ok(())
            //// Doesn't reach as we end conversation in the above call
            // could add a delay when handling the twilio websocket stop message?
            //let result = ClientToolResult::new(tool_call.client_tool_call.id())
            //    .is_error(false)
            //    .with_result(conf_name.clone());

            //agent_ws.lock().await.send_tool_result(result).await
            //    .map_err(|_| "Error sending tool result".to_string())?;
        }
        ToolName::PutHumanOperatorInConference => {
            info!("Tool Execution: PutHumanOperatorInConference");

            let tool_params = tool_call.client_tool_call.parameters();

            let conf_name = tool_params
                .get("conference_friendly_name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim_matches('"').to_string())
                .ok_or("Missing conference name".to_string())?;

            let twiml = create_conference_twiml(
                &conf_name,
                LABEL_PARTICIPANT_B,
                true,
                true,
                &state.config.https_ngrok_base_url,
            )?;

            // Check if the conference exists before adding the participant
            let is_existing = {
                let guard = state.conference_state.lock().await;
                guard.get_conference(&conf_name).await.is_some()
            };

            if !is_existing {
                return Err("Conference ended or caller left. Cannot join".into());
                warn!(
                    "Conference {} not found, Caller left before Participant B: {} could join ",
                    conf_name, tool_call.call_sid
                );
            }

            twilio_c
                .update_call_with_twiml(&tool_call.call_sid, &twiml)
                .await
                .map_err(|_| {
                    format!(
                        "Failed to add Participant B: {} to conference {}",
                        tool_call.call_sid, conf_name
                    )
                })?;
            Ok(())
        }
        ToolName::Unknown(name) => {
            warn!("Tool Execution: Unknown tool {}", name);
            Err(format!("Unknown tool {}", name))
        }
    }
}

fn create_conference_twiml(
    name: &str,
    label: &str,
    start_on_enter: bool,
    end_on_exit: bool,
    base_url: &str,
) -> Result<String, String> {
    let mut conference = Conference::new(name.to_string());
    conference.participant_label = Some(label.to_string());
    conference.start_conference_on_enter = Some(start_on_enter);
    conference.end_conference_on_exit = Some(end_on_exit);
    conference.status_callback = Some(format!("{}{}", base_url, PATH_EVENTS_CONFERENCE));
    conference.status_callback_event = Some("start end join leave".to_string());
    // Default hold music started to delay so using this for a while
    conference.wait_url = Some("https://twimlets.com/holdmusic?Bucket=com.twilio.music.ambient".to_string());
    VoiceResponse::new()
        .dial(conference)
        .to_string()
        .map_err(|e| e.to_string())
}
