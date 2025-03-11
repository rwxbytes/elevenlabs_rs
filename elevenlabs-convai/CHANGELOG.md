# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]
### Added
- `termination_reason` to `conversation::Metadata`
### Changed
- **Breaking**: `ElevenLabsAgentClient` name to `AgentWebSocket`
- **Breaking**: `start_conversation` method to `start_session` (perhaps not the methods)
- **Breaking**: `stop_conversation` method to `end_session`
## [0.2.0] - 2025-02-28
### Added
- dependency on elevenlabs_rs 0.5.1
## [0.1.0] - 2025-02-06
### Added
- Initial release as part of workspace
- Integration with elevenlabs_rs 0.4.0
