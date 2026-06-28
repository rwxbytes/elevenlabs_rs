# OpenAPI coverage

Generated: 2026-06-28T20:42:48Z
Snapshot: `openapi/elevenlabs-openapi.paths.json`
Source: https://api.elevenlabs.io/openapi.json

## Summary

| Metric | Value |
| --- | ---: |
| OpenAPI operations | 320 |
| Implemented method/path pairs | 269 |
| Coverage | 84.1% |
| Local endpoint constants checked | 269 |
| Local constants missing from snapshot | 0 |

## Coverage By Path

| Path | Implemented | Total | Coverage |
| --- | ---: | ---: | ---: |
| /docs | 0 | 1 | 0.0% |
| /v1/audio-isolation | 4 | 4 | 100.0% |
| /v1/audio-native | 4 | 4 | 100.0% |
| /v1/convai | 148 | 149 | 99.3% |
| /v1/dubbing | 6 | 20 | 30.0% |
| /v1/forced-alignment | 1 | 1 | 100.0% |
| /v1/history | 5 | 5 | 100.0% |
| /v1/models | 1 | 1 | 100.0% |
| /v1/music | 7 | 7 | 100.0% |
| /v1/productions | 0 | 11 | 0.0% |
| /v1/pronunciation-dictionaries | 9 | 9 | 100.0% |
| /v1/service-accounts | 5 | 5 | 100.0% |
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
| /v1/voices | 24 | 25 | 96.0% |
| /v1/workspace | 22 | 22 | 100.0% |
| /v1/workspaces | 2 | 2 | 100.0% |
| /v2/voices | 1 | 1 | 100.0% |

## Local Endpoint Constants Missing From Snapshot

All local endpoint method/path constants match the snapshot.

## Coverage Notes

The official OpenAPI snapshot still includes a few legacy or deprecated
operations. They remain visible in the generated coverage totals, but they are
not planned as `0.7.0` typed endpoints unless ElevenLabs reintroduces them as
current APIs.

- `GET /v1/convai/conversation/get_signed_url` is the legacy underscore-path
  signed URL route. The current hyphen-path route,
  `GET /v1/convai/conversation/get-signed-url`, is already implemented.
- `GET /v1/voices` is the legacy V1 voice-list route. The crate already exposes
  the current V2 voice-list variant.
- Dubbing paths containing `/resource/` are legacy beta-era dubbing resource
  routes and are intentionally left out of the `0.7.0` typed surface.

## OpenAPI Operations Not Yet Implemented

### /docs

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/docs` | Redirect To Mintlify |

### /v1/convai

Note: the missing signed URL operation below is the legacy underscore-path alias.
The current hyphen-path endpoint is already implemented.

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/convai/conversation/get_signed_url` | Get Signed Url |

### /v1/dubbing

Note: paths containing `/resource/` are legacy beta-era dubbing APIs and are not
planned for the `0.7.0` typed surface. The non-resource rows remain ordinary
coverage gaps.

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

Note: the missing operation below is the legacy V1 voice-list route. The crate
already exposes the current V2 voice-list variant.

| Method | Path | Summary |
| --- | --- | --- |
| `GET` | `/v1/voices` | List Voices |

## Maintainer Commands

```powershell
nu tools/openapi_coverage.nu fetch
nu tools/openapi_coverage.nu report
nu tools/openapi_coverage.nu check
```
