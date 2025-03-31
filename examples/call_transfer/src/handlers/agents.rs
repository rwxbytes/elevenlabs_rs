use crate::{
    helpers::send_tool_error, toolkit, AppState, ASSESSMENT_AGENT,
    WAIT_MANAGEMENT_AGENT, WARM_TRANSFER_AGENT,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use elevenlabs_twilio::TelephonyAgent;
use tracing::{error, field, info, instrument, Instrument, Span};

/// Handles the common WebSocket logic for both inbound and outbound agents.
///
/// Takes the application state, a mutable telephony agent, and the agent type as parameters.
/// Sets up the agent's WebSocket, spawns a tool handling task, and processes the phone call.
async fn handle_agent_websocket(
    state: AppState,
    mut agent: TelephonyAgent,
    agent_type: &str,
) -> Response {
    info!("WebSocket upgrade request for agent");

    if let Err(e) = agent.set_agent_ws(agent_type).await {
        error!(error = ?e, "Error setting agent websocket");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    info!("Agent WebSocket selected from registry");

    let (tools_tx, mut tools_rx) = tokio::sync::mpsc::unbounded_channel();
    agent.tools_tx = Some(tools_tx);

    let agent_ws_clone = agent.agent_ws.clone().unwrap();
    let twilio_c_clone = agent.twilio_client.clone();
    let app_state_clone = state.clone();

    // Spawn a background task to handle tool calls with its own span
    let tool_handler_task_span = tracing::info_span!("tool_task", agent_type = agent_type);

    tokio::spawn(
        async move {
            info!("Starting tool call handler task");
            while let Some(phone_call_tool) = tools_rx.recv().await {
                // Create a span for each tool call iteration
                let tool_iteration_span = tracing::info_span!(
                    "handle_one_tool",
                    tool.name = %phone_call_tool.client_tool_call.name(),
                    call_sid = %phone_call_tool.call_sid,
                    convo.id = ?phone_call_tool.conversation_id
                );

                // Define the async block and immediately instrument and await it
                async {
                    info!("Received tool call request");
                    match toolkit::handle_tool_call(
                        &phone_call_tool,
                        &agent_ws_clone,
                        &twilio_c_clone,
                        &app_state_clone,
                    )
                    .await
                    {
                        Ok(_) => info!("Tool call handled successfully"),
                        Err(e) => {
                            error!(error = ?e, "Tool call handling failed");
                            send_tool_error(
                                &agent_ws_clone,
                                phone_call_tool.client_tool_call.id(),
                                &format!("Tool call failed: {:?}", e), // make generic or better msg for agent
                            )
                            .instrument(Span::current())
                            .await;
                        }
                    }
                }
                .instrument(tool_iteration_span)
                .await;
            }
        }
        .instrument(tool_handler_task_span),
    );

    // Handle the phone call and return the response
    match agent.handle_phone_call().await {
        Ok(response) => {
            info!("WebSocket upgrade response generated");
            response
        }
        Err(e) => {
            error!(error = ?e, "Error during WebSocket handling setup");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Handler for inbound agent WebSocket connections.
#[instrument(
    name = "assessment_agent_ws",
    skip_all,
    fields(
        agent_type = ASSESSMENT_AGENT,
        //call_sid = field::Empty,
        //stream_sid = field::Empty,
    )
)]
pub(crate) async fn inbound_agent_handler(
    State(state): State<AppState>,
    agent: TelephonyAgent,
) -> Response {
    handle_agent_websocket(state, agent, ASSESSMENT_AGENT).await
}

/// Handler for outbound agent WebSocket connections.
#[instrument(
    name = "warm_transfer_agent_ws",
    skip_all,
    fields(
        agent_type = WARM_TRANSFER_AGENT,
        //call_sid = field::Empty,
        //stream_sid = field::Empty,
    )
)]
pub(crate) async fn outbound_agent_handler(
    State(state): State<AppState>,
    agent: TelephonyAgent,
) -> Response {
    handle_agent_websocket(state, agent, WARM_TRANSFER_AGENT).await
}

// Handler for another inbound agent WebSocket connection to notify the caller
#[instrument(
    name = "wait_manager_ws",
    skip_all,
    fields(
        agent_type = WAIT_MANAGEMENT_AGENT,
        //call_sid = field::Empty,
        //stream_sid = field::Empty,
    )
)]
pub(crate) async fn wait_management_agent_handler(
    State(state): State<AppState>,
    agent: TelephonyAgent,
) -> Response {
    handle_agent_websocket(state, agent, WAIT_MANAGEMENT_AGENT).await
}
