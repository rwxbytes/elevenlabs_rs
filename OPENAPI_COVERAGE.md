# OpenAPI coverage

Generated: 2026-06-25T23:55:38Z
Snapshot: `openapi/elevenlabs-openapi.paths.json`
Source: https://api.elevenlabs.io/openapi.json

## Summary

| Metric | Value |
| --- | ---: |
| OpenAPI operations | 320 |
| Implemented method/path pairs | 122 |
| Coverage | 38.1% |
| Local endpoint constants checked | 122 |
| Local constants missing from snapshot | 0 |

## Coverage By Path

| Path | Implemented | Total | Coverage |
| --- | ---: | ---: | ---: |
| /docs | 0 | 1 | 0.0% |
| /v1/audio-isolation | 4 | 4 | 100.0% |
| /v1/audio-native | 4 | 4 | 100.0% |
| /v1/convai | 38 | 149 | 25.5% |
| /v1/dubbing | 6 | 20 | 30.0% |
| /v1/forced-alignment | 1 | 1 | 100.0% |
| /v1/history | 5 | 5 | 100.0% |
| /v1/models | 1 | 1 | 100.0% |
| /v1/music | 7 | 7 | 100.0% |
| /v1/productions | 0 | 11 | 0.0% |
| /v1/pronunciation-dictionaries | 9 | 9 | 100.0% |
| /v1/service-accounts | 0 | 5 | 0.0% |
| /v1/shared-voices | 1 | 1 | 100.0% |
| /v1/similar-voices | 1 | 1 | 100.0% |
| /v1/single-use-token | 1 | 1 | 100.0% |
| /v1/sound-generation | 1 | 1 | 100.0% |
| /v1/speech-engine | 5 | 5 | 100.0% |
| /v1/speech-to-speech | 2 | 2 | 100.0% |
| /v1/speech-to-text | 3 | 3 | 100.0% |
| /v1/studio | 0 | 23 | 0.0% |
| /v1/text-to-dialogue | 4 | 4 | 100.0% |
| /v1/text-to-speech | 4 | 4 | 100.0% |
| /v1/text-to-voice | 5 | 5 | 100.0% |
| /v1/usage | 1 | 1 | 100.0% |
| /v1/user | 2 | 2 | 100.0% |
| /v1/voices | 10 | 25 | 40.0% |
| /v1/workspace | 6 | 22 | 27.3% |
| /v1/workspaces | 0 | 2 | 0.0% |
| /v2/voices | 1 | 1 | 100.0% |

## Local Endpoint Constants Missing From Snapshot

All local endpoint method/path constants match the snapshot.

## OpenAPI Operations Not Yet Implemented

### /docs

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/docs` | Redirect To Mintlify |

### /v1/convai

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/convai/agents/{agent_id}/topics` | Get Agent Conversation Topics |
| `POST` | `/v1/convai/agent-testing/bulk-move` | Bulk Move Tests To Folder |
| `POST` | `/v1/convai/agent-testing/folders` | Create Agent Test Folder |
| `DELETE` | `/v1/convai/agent-testing/folders/{folder_id}` | Delete Agent Test Folder |
| `GET` | `/v1/convai/agent-testing/folders/{folder_id}` | Get Agent Test Folder By Id |
| `PATCH` | `/v1/convai/agent-testing/folders/{folder_id}` | Update Agent Test Folder |
| `GET` | `/v1/convai/agent/{agent_id}/knowledge-base/size` | Returns The Size Of The Agent'S Knowledge Base |
| `POST` | `/v1/convai/agent/{agent_id}/llm-usage/calculate` | Calculate Expected Llm Usage For An Agent |
| `GET` | `/v1/convai/agents/summaries` | Get Agent Summaries |
| `GET` | `/v1/convai/agents/{agent_id}/branches` | List Agent Branches |
| `POST` | `/v1/convai/agents/{agent_id}/branches` | Create A New Branch |
| `GET` | `/v1/convai/agents/{agent_id}/branches/{branch_id}` | Get Agent Branch |
| `PATCH` | `/v1/convai/agents/{agent_id}/branches/{branch_id}` | Update Agent Branch |
| `POST` | `/v1/convai/agents/{agent_id}/branches/{branch_id}/rebase` | Rebase A Branch Onto Main |
| `GET` | `/v1/convai/agents/{agent_id}/branches/{branch_id}/rebase-preview` | Preview Rebased Configuration |
| `POST` | `/v1/convai/agents/{agent_id}/branches/{source_branch_id}/merge` | Merge A Branch Into A Target Branch |
| `GET` | `/v1/convai/agents/{agent_id}/branches/{source_branch_id}/merge-preview` | Preview Merged Configuration |
| `POST` | `/v1/convai/agents/{agent_id}/deployments` | Create Or Update Deployments |
| `DELETE` | `/v1/convai/agents/{agent_id}/drafts` | Delete Agent Draft |
| `POST` | `/v1/convai/agents/{agent_id}/drafts` | Create Agent Draft |
| `POST` | `/v1/convai/agents/{agent_id}/duplicate` | Duplicate Agent |
| `POST` | `/v1/convai/agents/{agent_id}/simulate-conversation` | Simulates A Conversation |
| `POST` | `/v1/convai/agents/{agent_id}/simulate-conversation/stream` | Simulates A Conversation (Stream) |
| `GET` | `/v1/convai/agents/{agent_id}/versions/{version_id}` | Get Agent Version Metadata |
| `GET` | `/v1/convai/analytics/live-count` | Get Live Count |
| `POST` | `/v1/convai/batch-calling/submit` | Submit A Batch Call Request. |
| `GET` | `/v1/convai/batch-calling/workspace` | Get All Batch Calls For A Workspace. |
| `DELETE` | `/v1/convai/batch-calling/{batch_id}` | Delete A Batch Call. |
| `GET` | `/v1/convai/batch-calling/{batch_id}` | Get A Batch Call By Id. |
| `POST` | `/v1/convai/batch-calling/{batch_id}/cancel` | Cancel A Batch Call. |
| `POST` | `/v1/convai/batch-calling/{batch_id}/retry` | Retry A Batch Call. |
| `GET` | `/v1/convai/conversation/get-signed-url` | Get Signed Url |
| `GET` | `/v1/convai/conversation/token` | Get Webrtc Token |
| `GET` | `/v1/convai/conversations/messages/smart-search` | Smart Search Conversation Messages |
| `GET` | `/v1/convai/conversations/messages/text-search` | Text Search Conversation Messages |
| `POST` | `/v1/convai/conversations/{conversation_id}/files` | Upload File |
| `DELETE` | `/v1/convai/conversations/{conversation_id}/files/{file_id}` | Delete File Upload |
| `GET` | `/v1/convai/conversations/{conversation_id}/sip-messages` | Get Sip Messages For A Conversation |
| `POST` | `/v1/convai/conversations/{conversation_id}/tags` | Assign Conversation Tags |
| `DELETE` | `/v1/convai/conversations/{conversation_id}/tags/{tag_id}` | Unassign Conversation Tag |
| `GET` | `/v1/convai/environment-variables` | List Environment Variables |
| `POST` | `/v1/convai/environment-variables` | Create Environment Variable |
| `GET` | `/v1/convai/environment-variables/{env_var_id}` | Get Environment Variable |
| `PATCH` | `/v1/convai/environment-variables/{env_var_id}` | Update Environment Variable |
| `POST` | `/v1/convai/exotel/outbound-call` | Handle An Outbound Call Via Exotel |
| `POST` | `/v1/convai/knowledge-base/file` | Create File Document |
| `GET` | `/v1/convai/knowledge-base/rag-index` | Get Rag Index Overview. |
| `POST` | `/v1/convai/knowledge-base/rag-index` | Compute Rag Indexes In Batch |
| `GET` | `/v1/convai/knowledge-base/search` | Search Knowledge Base Content |
| `GET` | `/v1/convai/knowledge-base/summaries` | Get Knowledge Base Summaries By Ids |
| `POST` | `/v1/convai/knowledge-base/text` | Create Text Document |
| `POST` | `/v1/convai/knowledge-base/url` | Create Url Document |
| `PATCH` | `/v1/convai/knowledge-base/{documentation_id}` | Update Document |
| `GET` | `/v1/convai/knowledge-base/{documentation_id}/chunks` | Get All Rag Chunks For A Document |
| `GET` | `/v1/convai/knowledge-base/{documentation_id}/rag-index` | Get Rag Indexes Of The Specified Knowledgebase Document. |
| `DELETE` | `/v1/convai/knowledge-base/{documentation_id}/rag-index/{rag_index_id}` | Delete Rag Index. |
| `POST` | `/v1/convai/knowledge-base/{documentation_id}/refresh` | Refresh Url Document Content |
| `GET` | `/v1/convai/knowledge-base/{documentation_id}/source-file-url` | Get Document Source File Url |
| `PATCH` | `/v1/convai/knowledge-base/{documentation_id}/update-file` | Update File Document |
| `POST` | `/v1/convai/llm-usage/calculate` | Calculate Expected Llm Usage |
| `GET` | `/v1/convai/llm/list` | List Available Llms |
| `GET` | `/v1/convai/mcp-servers` | List Mcp Servers |
| `POST` | `/v1/convai/mcp-servers` | Create Mcp Server |
| `DELETE` | `/v1/convai/mcp-servers/{mcp_server_id}` | Delete Mcp Server |
| `GET` | `/v1/convai/mcp-servers/{mcp_server_id}` | Get Mcp Server |
| `PATCH` | `/v1/convai/mcp-servers/{mcp_server_id}` | Update Mcp Server Configuration |
| `PATCH` | `/v1/convai/mcp-servers/{mcp_server_id}/approval-policy` | Update Mcp Server Approval Policy |
| `POST` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-approvals` | Create Mcp Server Tool Approval |
| `DELETE` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-approvals/{tool_name}` | Delete Mcp Server Tool Approval |
| `POST` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-configs` | Create Mcp Tool Configuration Override |
| `DELETE` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}` | Delete Mcp Tool Configuration Override |
| `GET` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}` | Get Mcp Tool Configuration Override |
| `PATCH` | `/v1/convai/mcp-servers/{mcp_server_id}/tool-configs/{tool_name}` | Update Mcp Tool Configuration Override |
| `GET` | `/v1/convai/mcp-servers/{mcp_server_id}/tools` | List Mcp Server Tools |
| `GET` | `/v1/convai/phone-numbers/{phone_number_id}/sip-messages` | Get Sip Messages For A Phone Number |
| `GET` | `/v1/convai/secrets/{secret_id}` | Get Convai Workspace Secret |
| `PATCH` | `/v1/convai/secrets/{secret_id}` | Update Convai Workspace Secret |
| `GET` | `/v1/convai/secrets/{secret_id}/dependencies/{resource_type}` | Get Secret Dependencies By Type |
| `GET` | `/v1/convai/settings/dashboard` | Get Convai Dashboard Settings |
| `PATCH` | `/v1/convai/settings/dashboard` | Update Convai Dashboard Settings |
| `POST` | `/v1/convai/sip-trunk/outbound-call` | Handle An Outbound Call Via Sip Trunk |
| `GET` | `/v1/convai/tags` | List Conversation Tags |
| `POST` | `/v1/convai/tags` | Create Conversation Tag |
| `DELETE` | `/v1/convai/tags/{tag_id}` | Delete Conversation Tag |
| `GET` | `/v1/convai/tags/{tag_id}` | Get Conversation Tag |
| `PATCH` | `/v1/convai/tags/{tag_id}` | Update Conversation Tag |
| `GET` | `/v1/convai/tools/{tool_id}/dependent-agents` | Get Dependent Agents List |
| `GET` | `/v1/convai/tools/{tool_id}/executions` | Get Tool Executions |
| `POST` | `/v1/convai/twilio/register-call` | Register A Twilio Call And Return Twiml |
| `GET` | `/v1/convai/users` | Get Conversation Users |
| `GET` | `/v1/convai/whatsapp-accounts` | List Whatsapp Accounts |
| `DELETE` | `/v1/convai/whatsapp-accounts/{phone_number_id}` | Delete Whatsapp Account |
| `GET` | `/v1/convai/whatsapp-accounts/{phone_number_id}` | Get Whatsapp Account |
| `PATCH` | `/v1/convai/whatsapp-accounts/{phone_number_id}` | Update Whatsapp Account |
| `POST` | `/v1/convai/whatsapp/outbound-call` | Make An Outbound Call Via Whatsapp |
| `POST` | `/v1/convai/whatsapp/outbound-message` | Send An Outbound Message Via Whatsapp |
| `POST` | `/v1/convai/conversations/{conversation_id}/analysis/evaluations/run` | Run Conversation Evaluation |
| `POST` | `/v1/convai/conversations/{conversation_id}/analysis/run` | Run Conversation Analysis |
| `POST` | `/v1/convai/knowledge-base/bulk-move` | Bulk Move Entities To Folder |
| `POST` | `/v1/convai/knowledge-base/folder` | Create Folder |
| `POST` | `/v1/convai/knowledge-base/{document_id}/move` | Move Entity To Folder |
| `GET` | `/v1/convai/agent-testing` | List Agent Response Tests |
| `POST` | `/v1/convai/agent-testing/create` | Create Agent Response Test |
| `POST` | `/v1/convai/agent-testing/summaries` | Get Agent Response Test Summaries By Ids |
| `DELETE` | `/v1/convai/agent-testing/{test_id}` | Delete Agent Response Test |
| `GET` | `/v1/convai/agent-testing/{test_id}` | Get Agent Response Test By Id |
| `PUT` | `/v1/convai/agent-testing/{test_id}` | Update Agent Response Test |
| `POST` | `/v1/convai/agents/{agent_id}/run-tests` | Run Tests On The Agent |
| `GET` | `/v1/convai/test-invocations` | List Test Invocations |
| `GET` | `/v1/convai/test-invocations/{test_invocation_id}` | Get Test Invocation |
| `POST` | `/v1/convai/test-invocations/{test_invocation_id}/resubmit` | Resubmit Tests |

### /v1/dubbing

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/dubbing` | List Dubs |
| `GET` | `/v1/dubbing/resource/{dubbing_id}` | Get The Dubbing Resource For An Id. |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/dub` | Dubs All Or Some Segments And Languages |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/migrate-segments` | Move Segments Between Speakers |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/render/{language}` | Render Audio Or Video For The Given Language |
| `DELETE` | `/v1/dubbing/resource/{dubbing_id}/segment/{segment_id}` | Deletes A Single Segment |
| `PATCH` | `/v1/dubbing/resource/{dubbing_id}/segment/{segment_id}/{language}` | Modify A Single Segment |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/speaker` | Create A New Speaker |
| `PATCH` | `/v1/dubbing/resource/{dubbing_id}/speaker/{speaker_id}` | Update Metadata For A Speaker |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/speaker/{speaker_id}/segment` | Create A Segment For The Speaker |
| `GET` | `/v1/dubbing/resource/{dubbing_id}/speaker/{speaker_id}/similar-voices` | Search The Elevenlabs Library For Voices Similar To A Speaker. |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/transcribe` | Transcribes Segments |
| `POST` | `/v1/dubbing/resource/{dubbing_id}/translate` | Translates All Or Some Segments And Languages |
| `GET` | `/v1/dubbing/{dubbing_id}/transcripts/{language_code}/format/{format_type}` | Retrieve A Transcript |

### /v1/productions

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/productions/orders` | List Orders |
| `POST` | `/v1/productions/orders` | Create Order |
| `GET` | `/v1/productions/orders/languages/{order_item_kind}` | Get Available Languages |
| `GET` | `/v1/productions/orders/{order_id}` | Get Order |
| `PATCH` | `/v1/productions/orders/{order_id}` | Update Order |
| `GET` | `/v1/productions/orders/{order_id}/deliverables` | Get Order Deliverables |
| `POST` | `/v1/productions/orders/{order_id}/items` | Upsert Order Item |
| `DELETE` | `/v1/productions/orders/{order_id}/items/{item_id}` | Remove Order Item |
| `POST` | `/v1/productions/orders/{order_id}/media` | Register Media |
| `GET` | `/v1/productions/orders/{order_id}/media/{media_id}` | Get Media Info |
| `POST` | `/v1/productions/orders/{order_id}/submit` | Submit Order |

### /v1/service-accounts

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/service-accounts` | Get Workspace Service Accounts |
| `GET` | `/v1/service-accounts/{service_account_user_id}/api-keys` | Get Service Account Api Keys Route |
| `POST` | `/v1/service-accounts/{service_account_user_id}/api-keys` | Create Service Account Api Key |
| `DELETE` | `/v1/service-accounts/{service_account_user_id}/api-keys/{api_key_id}` | Delete Service Account Api Key |
| `PATCH` | `/v1/service-accounts/{service_account_user_id}/api-keys/{api_key_id}` | Edit Service Account Api Key |

### /v1/studio

| Method | Path | Summary |
| --- | --- | --- |
| `POST` | `/v1/studio/podcasts` | Create Podcast |
| `GET` | `/v1/studio/projects` | List Studio Projects |
| `POST` | `/v1/studio/projects` | Create Studio Project |
| `DELETE` | `/v1/studio/projects/{project_id}` | Delete Studio Project |
| `GET` | `/v1/studio/projects/{project_id}` | Get Studio Project |
| `POST` | `/v1/studio/projects/{project_id}` | Update Studio Project |
| `GET` | `/v1/studio/projects/{project_id}/chapters` | List Chapters |
| `POST` | `/v1/studio/projects/{project_id}/chapters` | Create Chapter |
| `DELETE` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}` | Delete Chapter |
| `GET` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}` | Get Chapter |
| `POST` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}` | Update Chapter |
| `POST` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}/convert` | Convert Chapter |
| `GET` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}/snapshots` | List Chapter Snapshots |
| `GET` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}/snapshots/{chapter_snapshot_id}` | Get Chapter Snapshot |
| `POST` | `/v1/studio/projects/{project_id}/chapters/{chapter_id}/snapshots/{chapter_snapshot_id}/stream` | Stream Chapter Audio |
| `POST` | `/v1/studio/projects/{project_id}/content` | Update Studio Project Content |
| `POST` | `/v1/studio/projects/{project_id}/convert` | Convert Studio Project |
| `GET` | `/v1/studio/projects/{project_id}/muted-tracks` | Get Project Muted Tracks |
| `POST` | `/v1/studio/projects/{project_id}/pronunciation-dictionaries` | Create Pronunciation Dictionaries |
| `GET` | `/v1/studio/projects/{project_id}/snapshots` | List Studio Project Snapshots |
| `GET` | `/v1/studio/projects/{project_id}/snapshots/{project_snapshot_id}` | Get Project Snapshot |
| `POST` | `/v1/studio/projects/{project_id}/snapshots/{project_snapshot_id}/archive` | Stream Archive With Studio Project Audio |
| `POST` | `/v1/studio/projects/{project_id}/snapshots/{project_snapshot_id}/stream` | Stream Studio Project Audio |

### /v1/voices

| Method | Path | Summary |
| --- | --- | --- |
| `POST` | `/v1/voices/pvc` | Create Pvc Voice |
| `POST` | `/v1/voices/pvc/{voice_id}` | Edit Pvc Voice |
| `GET` | `/v1/voices/pvc/{voice_id}/captcha` | Get Pvc Voice Captcha |
| `POST` | `/v1/voices/pvc/{voice_id}/captcha` | Verify Pvc Voice Captcha |
| `POST` | `/v1/voices/pvc/{voice_id}/samples` | Add Samples To Pvc Voice |
| `DELETE` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}` | Delete Pvc Voice Sample |
| `POST` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}` | Update Pvc Voice Sample |
| `GET` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}/audio` | Retrieve Voice Sample Audio |
| `POST` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}/separate-speakers` | Start Speaker Separation |
| `GET` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}/speakers` | Retrieve Speaker Separation Status |
| `GET` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}/speakers/{speaker_id}/audio` | Retrieve Separated Speaker Audio |
| `GET` | `/v1/voices/pvc/{voice_id}/samples/{sample_id}/waveform` | Retrieve Voice Sample Visual Waveform |
| `POST` | `/v1/voices/pvc/{voice_id}/train` | Run Pvc Training |
| `POST` | `/v1/voices/pvc/{voice_id}/verification` | Request Manual Verification |
| `GET` | `/v1/voices` | List Voices |

### /v1/workspace

| Method | Path | Summary |
| --- | --- | --- |
| `POST` | `/v1/workspace/analytics/query/usage-by-product-over-time` | Get Workspace Usage |
| `POST` | `/v1/workspace/analytics/requests` | List Api Requests |
| `GET` | `/v1/workspace/audit-logs` | Get Workspace Audit Logs |
| `GET` | `/v1/workspace/auth-connections` | Get Workspace Auth Connections |
| `POST` | `/v1/workspace/auth-connections` | Create Workspace Auth Connection |
| `DELETE` | `/v1/workspace/auth-connections/{auth_connection_id}` | Delete Workspace Auth Connection |
| `PATCH` | `/v1/workspace/auth-connections/{auth_connection_id}` | Update Workspace Auth Connection |
| `GET` | `/v1/workspace/groups` | Get All Groups |
| `GET` | `/v1/workspace/groups/search` | Search User Groups |
| `POST` | `/v1/workspace/groups/{group_id}/members` | Add Member To User Group |
| `POST` | `/v1/workspace/groups/{group_id}/members/remove` | Delete Member From User Group |
| `POST` | `/v1/workspace/invites/add-bulk` | Invite Multiple Users |
| `GET` | `/v1/workspace/webhooks` | List Workspace Webhooks |
| `POST` | `/v1/workspace/webhooks` | Create Workspace Webhook |
| `DELETE` | `/v1/workspace/webhooks/{webhook_id}` | Delete Workspace Webhook |
| `PATCH` | `/v1/workspace/webhooks/{webhook_id}` | Update Workspace Webhook |

### /v1/workspaces

| Method | Path | Summary |
| --- | --- | --- |
| `POST` | `/v1/workspaces/api-keys/disable` | Disable Api Key |
| `POST` | `/v1/workspaces/api-keys/third-party-disabling` | Set Workspace Third-Party Disabling Policy |

## Maintainer Commands

```powershell
nu tools/openapi_coverage.nu fetch
nu tools/openapi_coverage.nu report
nu tools/openapi_coverage.nu check
```
