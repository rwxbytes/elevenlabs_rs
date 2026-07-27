# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

## [0.4.2] - 2026-07-27

### Added
- `send_user_message` and `send_user_activity` methods on `AgentWebSocketSession` and `AgentWebSocket`.

## [0.4.1] - 2026-06-29

### Fixed
- Enabled `native-tls` on `tokio-tungstenite` so ConvAI `wss://` WebSocket connections work when `elevenlabs-convai` is used outside the workspace.

## [0.4.0] - 2026-06-29

### Breaking
- `AgentWebSocket::start_conversation` now returns `AgentWebSocketSession` instead of an anonymous stream.
- `AgentWebSocket` implementation fields are now private; configure with constructors and builder methods, and inspect state through read-only accessors.
- `writer_task_tx` now uses a bounded `tokio::sync::mpsc::Sender`.
- `ConvAIError::WebSocketError` now boxes its `tungstenite::Error` source.
- Minimum supported Rust version is now 1.85.

### Added
- `AgentWebSocketSession` with `close`, `abort`, `join`, `is_closed`, `send_tool_result`, and `send_context_update`.
- `AgentWebSocketOptions` for inbound and outbound WebSocket buffer sizes.
- `ConvAIError::ElevenLabs` for errors returned by the underlying `elevenlabs_rs` client.
- `ServerMessage::Unknown` for undocumented or newly added server events.
- Typed client messages for `Feedback`, `UserMessage`, `UserActivity`, and `MultimodalMessage`.
- Local WebSocket tests for audio sending, protocol ping/pong, tool results, context updates, close errors, and unknown events.

### Changed
- WebSocket reader, writer, and audio sender tasks now retain join handles through `AgentWebSocketSession`.
- WebSocket inbound and outbound queues are now bounded.
- Server messages now deserialize by the `type` discriminator instead of untagged shape matching.
- `tokio-tungstenite` updated to `0.29`.

### Fixed
- Workspace builds now use the local `elevenlabs_rs` crate instead of resolving a second registry copy.
- `stop_conversation`, protocol ping handling, and send helpers no longer unwrap missing or mismatched state.
- Signed URL REST calls now enable TLS when `elevenlabs-convai` depends on `elevenlabs_rs` without default features.
- Non-normal WebSocket closes now preserve close code and reason.
- Unexpected WebSocket frames now report the received frame kind.

## [0.3.0] - 2025-04-05
### Added
- `send_tool_result` to `AgentWebSocket`
- getters for `tool_call_id`, `tool_name`, and `parameters` on `ClientToolCall`
- `ContextualUpdate` in `client_message` module
- `send_context_update` to `AgentWebSocket`

### Changed
- **Breaking**: `ElevenLabsAgentClient` name to `AgentWebSocket`
- `with_is_error` to `has_error` on `ClientToolResult`

### Fixed
- `tool_call_id` on `ClientToolResult`, it was `client_tool_id`

## [0.2.0] - 2025-02-28
### Added
- dependency on elevenlabs_rs 0.5.1
## [0.1.0] - 2025-02-06
### Added
- Initial release as part of workspace
- Integration with elevenlabs_rs 0.4.0
