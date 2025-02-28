# Changelog
All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
### Security

## [0.4.1] - 2025-02-06
### Fixed
- Added configuration to ensure all feature-gated modules (including `convai`) are visible in the published documentation on docs.rs