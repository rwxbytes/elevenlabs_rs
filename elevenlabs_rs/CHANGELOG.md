# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2026-07-27

### Added
- Native `speech_engine_livekit` workbench example using egui, CPAL, LiveKit, and Rig/Ollama, with Speech Engine configuration editing, interruption-aware response streaming, transcripts, and latency telemetry.
- Typed Speech Engine configuration and builders for supported voices, turn handling, file input, monitoring, background sound, privacy controls, and conversation-history redaction.
- Music detailed SSE streaming through `StreamMusicDetailed`, including decoded audio chunks, the `song-id` response header, metadata events, completion events, and forward-compatible unknown events.
- Music Finetune list, create, get, update, and delete endpoints, with multipart `FilePart` uploads and typed filters, visibility, creator, status, and failure-reason models.
- `ResolveConversation` and `QueryAgentKnowledgeBase` ConvAI endpoints.
- `CreateServiceAccount` and `GetWorkspaceMembers` admin endpoints.
- Music Finetune selection through `MusicComposeBody::with_finetune_id`.
- Speech-to-text `multichannel_output_style`, batch transcription token query support, and `SingleUseTokenType::batch_scribe`.
- MCP `ToolInterruptionMode` and environment selection for tool lookup, tool configuration, and MCP tool listing.
- Conversation product, status, termination-reason, analysis-score, reasoning, billing, audio-availability, branch, environment, tag, and trace fields from the current API schema.
- Numeric evaluation scoring configuration and the `gpt-5.6-sol`, `gpt-5.6-terra`, and `gpt-5.6-luna` LLM identifiers.
- Archived pronunciation dictionary filtering, workspace invite usage limits, workspace webhook event subscriptions, and service-account character-limit clearing.

### Changed
- `ConvoStatus` now preserves provider-owned status values instead of rejecting newly introduced values.
- Refreshed the OpenAPI snapshot to 352 operations; 279 local endpoint method/path pairs are implemented and all match the snapshot.
- Coverage reporting now distinguishes current Dubbing Project routes from legacy beta `/v1/dubbing/resource/...` routes.

### Deprecated
- `SimulateConversation` and `SimulateConversationStream`, matching the upstream deprecation in favor of agent tests.
- MCP `with_disable_interruptions` builders; use `with_interruption_mode`.
- `TTSConfig::with_optimize_streaming_latency` and `SpeechEngineTtsConfig::with_optimize_streaming_latency`, which ElevenLabs now treats as no-ops.

## [0.7.0] - 2026-06-29

### Breaking
- `elevenlabs_rs::Result` now uses the typed `elevenlabs_rs::error::Error` enum instead of a boxed trait-object error.
- `ElevenLabsEndpoint` is sealed. Downstream crates should use `ElevenLabsClient::raw` for endpoints not modeled by this crate yet.
- Default features are now slim and only enable `native-tls`. Enable `admin`, `genai`, `convai`, `ws`, and `playback` explicitly as needed.
- Minimum supported Rust version is now 1.85.
- `Alignment` timestamp fields and the `Timestamps` iterator item are now `f64` instead of `f32`.
- Model and format enums now preserve provider-owned values through open custom variants.

### Added
- Core client support for `ApiError`, `ApiResponse<T>`, and `ElevenLabsClient::hit_with_metadata`.
- `ElevenLabsClient::raw` for authenticated raw HTTP calls to endpoints not yet modeled by the crate.
- OpenAPI coverage snapshot/report tooling and endpoint contract tests for high-traffic REST and WebSocket surfaces.
- `FilePart` for multipart uploads from paths or in-memory bytes, plus in-memory upload constructors across speech-to-text, forced alignment, audio isolation, voice changer, dubbing, knowledge base, voices, similar voices, and widget avatars.
- Text-to-dialogue endpoints: `TextToDialogue`, `TextToDialogueWithTimestamps`, `TextToDialogueStream`, and `TextToDialogueStreamWithTimestamps`.
- Text-to-dialogue request/response helpers: `language_code`, `apply_text_normalization`, `TextToDialogueQuery::with_logging`, `TextToDialogueWithTimestampsResponse::{audio, segment_text, segments_with_text}`, and `VoiceSegment::duration`.
- Speech-core endpoints: `CreateSingleUseToken`, `CreateForcedAlignment`, `GetTranscript`, and `DeleteTranscript`.
- Speech-to-text additions: `CreateTranscript::with_query`, `SpeechToTextModel::ScribeV2`, and realtime speech-to-text WebSocket support through `RealtimeSpeechToText`.
- Text-to-voice endpoints: `TextToVoiceDesign`, `TextToVoiceRemix`, and `TextToVoicePreviewStream`.
- Text-to-speech WebSocket additions: `connect_text_to_speech`, `MultiContextWebSocketTTS`, `TTSWebSocketQuery::{with_sync_alignment, with_text_normalization, with_seed}`, `WebSocketSession<T>`, `WebSocketOptions`, and task completion reports.
- Speech Engine REST endpoints: `ListSpeechEngines`, `CreateSpeechEngine`, `GetSpeechEngine`, `UpdateSpeechEngine`, and `DeleteSpeechEngine`.
- Speech Engine upstream WebSocket protocol types, JWT verification, and a framework-neutral session wrapper.
- Music generation endpoints in `genai::music`: `ComposeMusic`, `StreamMusic`, `ComposeMusicDetailed`, `GenerateCompositionPlan`, `SeparateStems`, `UploadMusic`, and `VideoToMusic`.
- Music composition-plan types, including `MusicCompositionPlan`, `MusicPrompt`, `SongSection`, `SectionSource`, `TimeRange`, `CompositionPlan`, `CompositionChunk`, `GenerationChunk`, and `AudioRefChunk`.
- Audio isolation history endpoints: `GetAudioIsolationHistory` and `DeleteAudioIsolationHistoryItem`.
- Audio-native content endpoints: `UpdateAudioNativeContentFromUrl`, `UpdateAudioNativeProjectContent`, and `GetAudioNativeProjectSettings`.
- Workspace/admin endpoints for audit logs, groups, bulk invites, webhooks, service accounts, service-account API keys, workspace usage analytics, API request analytics, API-key disabling, and third-party-disabling policy.
- Workspace auth-connection endpoints in `admin::auth_connections`, with typed per-auth-type request builders and discriminated response models.
- Professional Voice Cloning endpoints in `admin::pvc_voices`, covering voice creation/editing, captcha verification, samples, speaker separation, training, and manual verification.
- Pronunciation dictionary endpoints `AddDictionaryFromRules`, `SetRules`, and `UpdateDictionary`, plus `WorkspaceAccess` and `permission_on_resource` response fields.
- Conversational AI endpoint modules for batch calling, conversation tags, environment variables, outbound telephony, LLM usage, WhatsApp accounts, test invocations, agent testing, agent management, and MCP servers.
- Conversational AI knowledge-base endpoints for text/URL/file documents, folders, bulk/move operations, RAG indexes, search, summaries, chunks, refresh, source-file URLs, file updates, metadata updates, and document RAG indexes.
- Conversational AI conversation-management endpoints for message search, analysis/evaluation runs, conversation file upload/delete, SIP messages, and tag assignment.
- Conversational AI workspace/tool/conversation additions including `GetSipMessages`, `GetSecretDependencies`, `GetDashboardSettings`, `UpdateDashboardSettings`, `GetSecret`, `UpdateSecret`, `GetToolDependentAgents`, `GetToolExecutions`, `GetWebRtcToken`, `RegisterTwilioCall`, `GetAgentKnowledgeBaseSize`, `GetLiveCount`, and `GetConversationUsers`.
- Builder methods on `GetSignedUrl`/`GetSignedUrlQuery` for custom query values, `include_conversation_id`, `branch_id`, and `environment`.
- `dialogue_karaoke` example.
- Public `accent` field on `VerifiedLanguage`.

### Changed
- `ConvAIModel::default()` now uses `ElevenFlashV2` instead of the deprecated `ElevenTurboV2`.
- `tokio` no longer uses the `full` feature set in normal builds.
- WebSocket transport dependencies are now gated behind the `ws` feature.
- WebSocket TTS and realtime STT now share a sealed internal endpoint/codec transport abstraction.
- WebSocket sessions now retain reader/writer task handles, use shared URL/auth construction, preserve unknown protocol fields, classify live STT error events, and report endpoint/direction context in frame/decode/close errors.
- Internal workspace dependencies now resolve to local workspace members during development.
- Request bodies are now attached for all HTTP methods when an endpoint provides one.

### Deprecated
- `ElevenLabsClient::hit_ws`; use `connect_text_to_speech`. It will be kept for one release.
- `GetUsage` (admin usage endpoint), matching the upstream deprecation. It remains available while ElevenLabs supports the endpoint.
- `CreateKnowledgeBaseDoc` (`POST /v1/convai/knowledge-base`), matching the upstream deprecation. Prefer `CreateFileDocument`, `CreateUrlDocument`, or `CreateTextDocument`; the legacy endpoint remains available while ElevenLabs supports it.
- Legacy model variants for `scribe_v1`, `eleven_monolingual_v1`, `eleven_multilingual_v1`, `eleven_turbo_v2`, and `eleven_turbo_v2_5`. They remain available while ElevenLabs accepts those model IDs.

### Removed
- Removed the custom `elevenlabs_twilio` bridge crate and Twilio server examples from this public workspace. Official ElevenLabs Twilio endpoints remain in `elevenlabs_rs`.

### Fixed
- `GetSignedUrl` now targets the current `/v1/convai/conversation/get-signed-url` path (previously the outdated `get_signed_url`).
- Removed the incorrect deprecation marker from `GetSignedUrl`; only the legacy `get_signed_url` alias is deprecated.
- Streaming-with-timestamps JSON parser (in `tts` and `text_to_dialogue`) now buffers across network chunk boundaries instead of assuming one chunk is exactly one message; `segment_text` offsets per-chunk character indices so it is correct for stream chunks.
- WebSocket TTS text chunks now serialize through `serde_json` instead of manual string formatting, so quotes, backslashes, control characters, and Unicode are escaped correctly.
- WebSocket TTS URL path/query construction now percent-encodes values correctly.
- WebSocket TTS BOS messages no longer serialize unset optional auth/config fields as JSON `null`.
- WebSocket TTS background task errors are now forwarded through the returned stream.
- WebSocket TTS audio decoding correctly handles `isFinal: false`.
- WebSocket TTS protocol error payloads now expose `code`, `error`, and `message`.
- Single-use token creation now sends an explicit zero-length request body so the API receives `Content-Length: 0`.
- Standalone feature builds, including `genai` without `admin`.
- Workspace check and clippy failures in examples and workspace members.
- Endpoint path parameters are now percent-encoded by URL path segment.
- Non-success HTTP responses now preserve status, headers, raw body, and parsed JSON error metadata.
- Query builders are now applied on TTS, TTS timestamp, text-to-voice, and speech-to-text endpoints that previously stored but ignored them.
- Current ConvAI phone-number, Twilio outbound-call, and text-to-voice save paths.
- `CreatePhoneNumberBody` now serializes to the flat API payload shape.
- `convai::tools` is exported again.

## [0.6.0] - 2025-04-05

### Added
- `ComputeRAGIndex` endpoint
- `prompt_injectable` field to `GetKnowledgeBaseDocResponse`, `CreateKnowledgeBaseDocResponse`, and `Document`
- `name` field to `CreateKnowledgeBaseDocBody`
- `speed` field to `TTSConfig` 
- LLM models
  - claude-3.7-sonnet
  - gemini-2.0-flash-lite
- `DeleteSecret` endpoint
- `termination_reason` to `conversation::Metadata`
- variant `DynamicVar::Null` to `DynamicVar`
- variant `Pcm8000hz` to `ConvAIAudioFormat`
- fields to `PromtpConfig`
  - `ignore_default_personality`
  - `rag`
- field `usage_mode` to `KnowledgeBase`
- field `dynamic_variables` to `Tool`
- field `workspace_overrides` to `PlatformSettings`
- fields to `Widget`
  - `show_page_show_terms`
  - `mic_muting_enabled`
- Opus format variants to `OutputFormat`
- fields to `GetKnowledgeBaseDocResponse`
  - `metadata`
  - `url`
- query `output_format` to `CreateSoundEffect`
- variant `SipTrunk` to `PhoneNumberProvider`
- query `enable_logging` to `CreateTranscript`
- field `phone_numbers` to `UsedBy`
- endpoints in `workspace` module:
  - `GetResource`
  - `ShareWorkspaceResource`
  - `UnshareWorkspaceResource`
- builder method `with_language_presets(HashMap<String, LanguagePreset>)` to `ConversationConfig`
- field `phone_call` to `GetConversationDetails`
- fields to `TextToVoiceBody`
  - `loudness`
  - `quality`
  - `seed`
  - `guidance_scale`
- endpoint `GetVoices` (uses the V2 API)
- a few queries to `GetVoices`
- endpoint `GetDefaultVoiceSettings`
- endpoint `OutboundCallViaTwilio`
- field `version_rules_num` to `CreateDictionaryResponse` and `RulesResponse`
- fields `latest_version_rules_num` and `archived_time_unix` to `DictionaryMetadataResponse`
- query methods `with_sort` and `with_sort_direction` to `GetDictionariesQuery`
- endpoints in `knowledge_base` module:
  - `GetDocumentContent`
  - `GetDocumentChunk`
- field `rag_retrieval_info` to `Transcript`
- variant `SipTrunk` to `CreatePhoneNumberBody`
- variant `Text` to `KnowledgeBaseDocType`
- field `additional_formats` to `CreateTranscriptBody` and `CreateTranscriptResponse`
- field `locale` to `SharedVoice`
- field `apply_language_text_normalization` to `TextToSpeechBody`
  

### Changed
- **Breaking**: `GetKnowledgeBase` to `GetKnowledgeBaseDoc`
- **Breaking**: `GetKnowledgeBaseResponse` to `GetKnowledgeBaseDocResponse`
- **Breaking**: `CreateKnowledgeBase` to `CreateKnowledgeBaseDoc`
- **Breaking**: `CreateKnowledgeBaseBody` to `CreateKnowledgeBaseDocBody`
- **Breaking**: `CreateKnowledgeBaseResponse` to `CreateKnowledgeBaseDocResponse`
-  `KnowledgeBaseType` to `KnowledgeBaseDocType`
- **Breaking**: `LLM::Gemini2_0FlashExp` to `LLM::Gemini2_0Flash001`
- **Breaking**: Moved `Secret`, `UsedBy`, `AgentTool`, and `SecretType` from `convai::agents` to `convai::workspace`
- The fields on `SharedVoice` are now optional:
  - `language`
  - `description`
  - `preview_url`
  - `rate`
- The fields on `Saftey` and types.
  - `ivc` to `is_blocked_ivc`
  - `non_ivc` to `is_blocked_non_ivc`
- `RagModel` to `EmbeddingModel`
- `TextToSoundEffects` to `CreateSoundEffect`
- `metadata` and `transcript` on `GetConversationDetails` are now optional
- `similarity_boost` and `stability` on `VoiceSettings` are now optional
- `CreatePhoneNumberBody` from a struct to an enum containing the following variants:
  - `CreatePhoneNumberBody::Twilio` 
  - `CreatePhoneNumberBody::SipTrunk` 


### Fixed
- The api key field on `CustomLLM` and its type
- `dynamic_variable_placeholders` field on `DynamicVariables`. The last letter was missing
- `access_level` field to `access_info` on `GetKnowledgeBaseDocResponse`

### Removed
- **Breaking:**`GetVoiceQuery` as now deprecated
- **Breaking:** `secrets` field from `GetAgentResponse` and `UpdateAgentBody`
- **Breaking:** `secrets` field from `GetSettingsResponse` and `UpdateSettingsBody`
- `knowledge_base_document_ids` field from `PromptConfig`
-  Commented out the `convai::tools` module for a while
- `ComputeRAGIndexQuery`

## [0.5.1] - 2025-02-28
### Added
- moved `SharedVoice` back to `voice_library` module
## [0.5.0] - 2025-02-28

### Added
- A `speech_to_text` module providing the following endpoint:
  - `CreateTranscript`
- A `tools` module providing the following endpoints:
  - `ListTools` to list all tools
  - `GetTool` to get a specific tool
  - `CreateTool` to create a new tool
  - `UpdateTool` to update an existing tool
  - `DeleteTool` to delete an existing tool
- `knowledge_base_document_ids` field to the `PromptConfig`
- `AgentQuery` on `CreateAgent` and `UpdateAgent`
  - `use_tool_ids` Use tool ids instead of tools specs from request payload.   
- `name` and `access_level` fields to `GetKnowledgeBaseResponse` struct
- `ListKnowledgeBasesDocs` endpoint
- `ListDependentAgents` endpoint
- `DeleteKnowledgeBaseDoc` endpoint
- A `convai::workspace` mod providing the following endpoints:
  - `GetSettings` to retrieve Convai settings for the workspace
  - `UpdateSettings` to update workspace settings
  - `GetSecrets` to list all workspace secrets
  - `CreateSecret` to create a new workspace secret
- `phone_numbers` field to `GetAgentResponse`
- `enable_conversation_initiation_client_data_from_webhook` field to `Overrides`
- `access_info` field to `Agent` on `GetAgentsResponse`
- Fields to `Widget`
  - `expandable`
  - `show_avatar_when_collapsed`
  - `disable_banner`
  - `language_selector`
- Query parameter builder methods to `KnowledgeBaseQuery`:
  - `with_search(search)`
  - `show_only_owned_documents()`
  - `use_typesense()`
- `verified_languages` to `GetVoiceResponse`
- `speed` field to `VoiceSettings`
- Fields to `SharedVoice`
  - `verified_languages`
  - `image_url`
  - `is_added_by_user`
- Query parameter builder method to `SharedVoiceQuery`:
  - `with_min_notice_period_days(days)`

### Changed
- **Breaking**: The `ResponseBody` of `DeleteAgent` now returns a `()` instead of a `StatusResponseBody`
- **Breaking**: The `used_tools` field name to `tool_ids` of the `PromptConfig` and its type `Option<Vec<UsedTool>>` 
  has been changed to `Option<Vec<String>>`. 
### Deprecated
### Removed
- The `UsedTool` struct
- **Breaking**: The field `supported_language_overrides` on `Widget`
### Fixed
- **Breaking** The `GetAgentResponse` by wrapping the `Vec<Secret>` in an `Option`
- The `GetVoiceResponse` by wrapping `VoiceVerification` optional fields in `Option`s 
- `ListPhoneNumbers`, it was missing the trailing slash in the path
### Security

## [0.4.1] - 2025-02-06
### Fixed
- Added configuration to ensure all feature-gated modules (including `convai`) are visible in the published documentation on docs.rs
