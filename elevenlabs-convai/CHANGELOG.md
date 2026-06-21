# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]
### Added
- `ConvAIError::ElevenLabs` for errors returned by the underlying `elevenlabs_rs` client

### Changed
- **Breaking**: `ConvAIError::WebSocketError` now boxes its `tungstenite::Error` source

### Fixed
- Workspace builds now use the local `elevenlabs_rs` crate instead of resolving a second registry copy

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
