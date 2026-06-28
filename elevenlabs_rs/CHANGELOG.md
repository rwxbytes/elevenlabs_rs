# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `TextToDialogue` endpoint
- `TextToDialogueWithTimestamps` endpoint
- `TextToDialogueStream` endpoint
- `TextToDialogueStreamWithTimestamps` endpoint
- `language_code` and `apply_text_normalization` fields to `TextToDialogueBody`
- `with_logging` query method to `TextToDialogueQuery`
- helper methods to `TextToDialogueWithTimestampsResponse`: `audio`, `segment_text`, `segments_with_text`
- `VoiceSegment::duration` method
- `dialogue_karaoke` example
- public `accent` field on `VerifiedLanguage`
- typed `ApiError`/`Error` model and `ApiResponse`
- `ElevenLabsClient::hit_with_metadata`
- endpoint contract tests for high-traffic TTS, STT, voices, history, pronunciation dictionary, ConvAI, and text-to-dialogue surfaces
- `CreateTranscript::with_query`
- `SpeechToTextModel::ScribeV2`
- WebSocket TTS serialization tests for escaped text/control messages
- OpenAPI coverage snapshot, report, and maintainer tooling
- `ElevenLabsClient::raw` for authenticated raw HTTP calls to endpoints not yet modeled by the crate
- public `WebSocketSession<T>` with `close`, `abort`, and `is_closed` lifecycle controls
- `ElevenLabsClient::connect_text_to_speech` and `ElevenLabsClient::connect_realtime_speech_to_text` as the preferred WebSocket session entry points
- `WebSocketOptions`, `WebSocketSession::join`, and WebSocket task completion reports
- local WebSocket integration tests for TTS/STT message order, auth, close, ping, non-normal close, and malformed JSON handling
- `CreateSingleUseToken` endpoint
- `CreateForcedAlignment` endpoint
- `GetTranscript` and `DeleteTranscript` endpoints
- `TextToVoiceDesign`, `TextToVoiceRemix`, and `TextToVoicePreviewStream` endpoints
- realtime speech-to-text WebSocket support via `RealtimeSpeechToText`
- multi-context text-to-speech WebSocket support via `MultiContextWebSocketTTS`
- `with_sync_alignment`, `with_text_normalization`, and `with_seed` methods to `TTSWebSocketQuery`
- `FilePart` for multipart uploads from either paths or in-memory bytes
- in-memory upload constructors for speech-to-text, forced alignment, audio isolation, voice changer, dubbing, knowledge base, voice, similar voice, and widget avatar bodies
- Speech Engine REST endpoints: `ListSpeechEngines`, `CreateSpeechEngine`, `GetSpeechEngine`, `UpdateSpeechEngine`, and `DeleteSpeechEngine`
- Speech Engine upstream WebSocket protocol types, JWT verification, and framework-neutral session wrapper
- Music generation endpoints in a new `genai::music` module: `ComposeMusic`, `StreamMusic`, `ComposeMusicDetailed`, `GenerateCompositionPlan`, `SeparateStems`, `UploadMusic`, and `VideoToMusic`
- Music composition-plan types: `MusicCompositionPlan`, `MusicPrompt`, `SongSection`, `SectionSource`, `TimeRange`, `CompositionPlan`, `CompositionChunk`, `GenerationChunk`, and `AudioRefChunk`
- Audio isolation history endpoints: `GetAudioIsolationHistory` and `DeleteAudioIsolationHistoryItem`
- Audio-native content endpoints: `UpdateAudioNativeContentFromUrl`, `UpdateAudioNativeProjectContent`, and `GetAudioNativeProjectSettings`
- Workspace endpoints: `GetWorkspaceAuditLogs`, `GetWorkspaceGroups`, `SearchWorkspaceGroups`, `AddMemberToGroup`, `RemoveMemberFromGroup`, `InviteUsers` (bulk), `GetWorkspaceWebhooks`, `CreateWorkspaceWebhook`, `UpdateWorkspaceWebhook`, and `DeleteWorkspaceWebhook`
- Conversational AI batch-calling endpoints in a new `convai::batch_calling` module: `SubmitBatchCall`, `ListWorkspaceBatchCalls`, `GetBatchCall`, `CancelBatchCall`, `RetryBatchCall`, and `DeleteBatchCall`
- Conversational AI endpoints completing several clusters: `GetSipMessages` (phone-numbers), `GetSecretDependencies`, `GetDashboardSettings`, `UpdateDashboardSettings` (workspace), `GetToolDependentAgents`, `GetToolExecutions` (tools), `GetWebRtcToken`, and `RegisterTwilioCall` (conversations)
- Conversational AI conversation-tags endpoints in a new `convai::tags` module: `ListConversationTags`, `CreateConversationTag`, `GetConversationTag`, `UpdateConversationTag`, and `DeleteConversationTag`
- Conversational AI environment-variable endpoints in a new `convai::environment_variables` module: `ListEnvironmentVariables`, `CreateEnvironmentVariable`, `GetEnvironmentVariable`, and `UpdateEnvironmentVariable`
- Conversational AI outbound telephony endpoints in a new `convai::telephony` module: `ExotelOutboundCall`, `SipTrunkOutboundCall`, `WhatsAppOutboundCall`, and `WhatsAppOutboundMessage`
- Conversational AI LLM endpoints in a new `convai::llm` module: `ListLlms`, `CalculateLlmUsage`, and `CalculateAgentLlmUsage`
- Conversational AI singleton endpoints: `GetAgentKnowledgeBaseSize` (agents), `GetLiveCount` (analytics), and `GetConversationUsers` (users)
- Conversational AI WhatsApp-account endpoints in a new `convai::whatsapp_accounts` module: `ListWhatsAppAccounts`, `GetWhatsAppAccount`, `UpdateWhatsAppAccount`, and `DeleteWhatsAppAccount`
- Conversational AI test-invocation endpoints in a new `convai::test_invocations` module: `ListTestInvocations`, `GetTestInvocation`, and `ResubmitTests`
- Conversational AI knowledge-base endpoints: `CreateTextDocument`, `CreateUrlDocument`, `CreateFileDocument`, `CreateKnowledgeBaseFolder`, `BulkMoveKnowledgeBase`, `MoveKnowledgeBaseEntity`, `GetRagIndexOverview`, `ComputeRagIndexesBatch`, `DeleteRagIndex`, `SearchKnowledgeBase`, `GetKnowledgeBaseSummaries`, `GetDocumentChunks`, `RefreshDocument`, `GetSourceFileUrl`, and `UpdateFileDocument`
- Conversational AI conversation-management endpoints: `SmartSearchConversationMessages`, `TextSearchConversationMessages`, `RunConversationAnalysis`, `RunConversationEvaluation`, `UploadConversationFile`, `DeleteConversationFile`, `GetConversationSipMessages`, `AssignConversationTags`, and `UnassignConversationTag`
- Conversational AI knowledge-base `UpdateKnowledgeBaseDocument` and `GetDocumentRagIndexes`, and workspace `GetSecret`/`UpdateSecret` endpoints (completing `/v1/convai` coverage)
- Advanced Conversational AI agent-management endpoints in a new `convai::agent_management` module: agent summaries; branch create/list/get/update/rebase/rebase-preview/merge/merge-preview; deployments; draft create/delete; `DuplicateAgent`; `RunAgentTests`; `SimulateConversation` and `SimulateConversationStream`; `GetAgentTopics`; and `GetAgentVersion`
- Conversational AI agent-testing endpoints in a new `convai::agent_testing` module: `ListAgentTests`, `CreateAgentTest`, `GetAgentTest`, `UpdateAgentTest`, `DeleteAgentTest`, `GetAgentTestSummaries`, `BulkMoveTests`, `CreateAgentTestFolder`, `GetAgentTestFolder`, `UpdateAgentTestFolder`, and `DeleteAgentTestFolder`
- Conversational AI MCP-server endpoints in a new `convai::mcp_servers` module: `CreateMcpServer`, `ListMcpServers`, `GetMcpServer`, `DeleteMcpServer`, `UpdateMcpServerConfig`, `UpdateMcpApprovalPolicy`, `AddMcpToolApproval`, `DeleteMcpToolApproval`, `CreateMcpToolConfig`, `GetMcpToolConfig`, `UpdateMcpToolConfig`, `DeleteMcpToolConfig`, and `ListMcpTools`
- `with_query`, `with_include_conversation_id`, `with_branch_id`, and `with_environment` builder methods to `GetSignedUrl`/`GetSignedUrlQuery`
- Workspace auth-connection endpoints in a new `admin::auth_connections` module: `CreateAuthConnection`, `ListAuthConnections`, `UpdateAuthConnection`, and `DeleteAuthConnection`, with typed per-auth-type request builders and a discriminated `AuthConnection`/`AuthConnectionConfig` response model
- Professional Voice Cloning (PVC) endpoints in a new `admin::pvc_voices` module: `CreatePvcVoice`, `EditPvcVoice`, `GetPvcVoiceCaptcha`, `VerifyPvcVoiceCaptcha`, `AddPvcVoiceSamples`, `UpdatePvcVoiceSample`, `DeletePvcVoiceSample`, `GetPvcSampleAudio`, `StartSpeakerSeparation`, `GetSpeakerSeparationStatus`, `GetSeparatedSpeakerAudio`, `GetPvcSampleWaveform`, `RunPvcTraining`, and `RequestPvcManualVerification`
- Pronunciation dictionary endpoints: `AddDictionaryFromRules`, `SetRules`, and `UpdateDictionary`
- `WorkspaceAccess` enum for pronunciation dictionary access levels
- `permission_on_resource` field to `CreateDictionaryResponse` and `DictionaryMetadataResponse`

### Changed
- **Breaking**: `elevenlabs_rs::Result` now uses `elevenlabs_rs::error::Error` instead of a boxed trait-object error
- **Breaking**: `ElevenLabsEndpoint` is now sealed; use `ElevenLabsClient::raw` for unsupported endpoints
- **Breaking**: `Alignment` timestamp fields and the `Timestamps` iterator item are now `f64` (were `f32`)
- **Breaking**: model and format enums now include open custom variants for provider-owned values
- Deprecated old ElevenLabs model variants and changed `ConvAIModel::default()` from `ElevenTurboV2` to `ElevenFlashV2`
- WebSocket TTS streams now retain reader/writer task handles and abort them when the returned stream is dropped
- WebSocket transport now uses shared URL/auth construction and more diagnostic close/frame/decode errors
- WebSocket TTS and realtime STT now route through a sealed internal endpoint/codec transport abstraction
- WebSocket protocol models now preserve unknown fields, realtime STT preserves unknown future events, and TTS audio decoding correctly handles `isFinal: false`
- WebSocket TTS protocol error payloads now expose `code`, `error`, and `message` fields
- Realtime STT WebSocket `invalid_request` and `input_error` events are now classified as protocol errors
- WebSocket frame/decode/close errors now include endpoint and message-direction context
- Internal workspace dependencies now resolve to local workspace members during development
- Request bodies are now attached for all HTTP methods when an endpoint provides one

### Deprecated
- `ElevenLabsClient::hit_ws`; use `connect_text_to_speech`. It will be kept for one release.
- `GetUsage` (admin usage endpoint). It will be kept for one release.
- `CreateKnowledgeBaseDoc` (`POST /v1/convai/knowledge-base`); use `CreateFileDocument`, `CreateUrlDocument`, or `CreateTextDocument`. It will be kept for one release.
- `GetSignedUrlLegacy` (`GET /v1/convai/conversation/get_signed_url`); use `GetSignedUrl`. It will be kept for one release.

### Fixed
- `GetSignedUrl` now targets the current `/v1/convai/conversation/get-signed-url` path (previously the outdated `get_signed_url`)
- Streaming-with-timestamps JSON parser (in `tts` and `text_to_dialogue`) now buffers across network chunk boundaries instead of assuming one chunk is exactly one message; `segment_text` offsets per-chunk character indices so it is correct for stream chunks
- WebSocket TTS text chunks now serialize through `serde_json` instead of manual string formatting, so quotes, backslashes, control characters, and Unicode are escaped correctly
- WebSocket TTS URL path/query construction now percent-encodes values correctly
- WebSocket TTS BOS messages no longer serialize unset optional auth/config fields as JSON `null`
- WebSocket TTS background task errors are now forwarded through the returned stream
- Single-use token creation now sends an explicit zero-length request body so the API receives `Content-Length: 0`
- Standalone feature builds, including `genai` without `admin`
- Workspace check and clippy failures in examples and integrations
- Endpoint path parameters are now percent-encoded by URL path segment
- Non-success HTTP responses now preserve status, headers, raw body, and parsed JSON error metadata
- Query builders are now applied on TTS, TTS timestamp, text-to-voice, and speech-to-text endpoints that previously stored but ignored them
- Current ConvAI phone-number, Twilio outbound-call, and text-to-voice save paths
- `CreatePhoneNumberBody` now serializes to the flat API payload shape
- `convai::tools` is exported again

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
