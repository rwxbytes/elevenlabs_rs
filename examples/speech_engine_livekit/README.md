# Speech Engine LiveKit workbench

This example is a native desktop workbench for ElevenLabs Speech Engine,
LiveKit, and a Rig/Ollama agent.

- ElevenLabs connects to `/ws`, the Speech Engine brain WebSocket.
- Rust streams transcript responses with Rig and Ollama.
- A Rust LiveKit bridge participant sends room audio to Speech Engine.
- The native desktop participant publishes your system microphone with CPAL.
- The native desktop participant plays agent audio with CPAL.
- The UI lets you switch Ollama models, edit the Rig system prompt, load,
  update, and create Speech Engines, adjust output gain, and inspect
  latency/debug events.

## Required setup

Configure the Speech Engine audio formats before running:

- `asr.user_input_audio_format = "pcm_16000"`
- `tts.agent_output_audio_format = "pcm_24000"`
- `conversation.client_events` should include `interruption`, `agent_response`,
  `agent_response_correction`, `agent_response_complete`, `user_transcript`, and `audio`

The example sets those values when it creates a Speech Engine. If you reuse an
existing `ELEVENLABS_SPEECH_ENGINE_ID`, update that engine in the dashboard or
API first.

Set:

```powershell
$env:ELEVENLABS_API_KEY="..."
$env:ELEVENLABS_SPEECH_ENGINE_ID="seng_..."
$env:LIVEKIT_URL="wss://your-project.livekit.cloud"
$env:LIVEKIT_API_KEY="..."
$env:LIVEKIT_API_SECRET="..."
```

If you want the example to create the Speech Engine for you, omit
`ELEVENLABS_SPEECH_ENGINE_ID` and expose this app's `/ws` route with a secure
public URL:

```powershell
$env:ELEVENLABS_SPEECH_ENGINE_WS_URL="wss://your-ngrok-host.ngrok-free.app/ws"
```

Optional:

```powershell
$env:LIVEKIT_ROOM="elevenlabs-rust-demo"
$env:LIVEKIT_EXAMPLE_BIND="127.0.0.1:3003"
$env:OLLAMA_MODEL="llama3.2:latest"
$env:OLLAMA_HOST="http://127.0.0.1:11434"
$env:SPEECH_ENGINE_VERIFY_AUTH="true"
```

`SPEECH_ENGINE_VERIFY_AUTH` defaults to `false` for this local/ngrok example so
tunnel setup and API-key mismatches do not reject the upstream WebSocket before
you can test the full audio path. Enable it for production-style testing, or put
the brain server behind equivalent network-level restrictions.

Run:

```powershell
cargo run -p speech_engine_livekit
```

The desktop window opens automatically. Click **Connect** to join the LiveKit
room with your system microphone and speakers.

The LiveKit bridge joins the room at startup, but the Speech Engine conversation
WebSocket is opened lazily when participant audio first reaches the bridge. This
keeps startup quiet and makes connection resets recoverable while you are using
the workbench.

## Ollama model picker

The model picker reads local Ollama models from:

```text
http://127.0.0.1:11434/api/tags
```

Set `OLLAMA_HOST` if Ollama is running elsewhere. The UI also links to the
Ollama model registry so you can pull and test different models.

## Audio notes

This workbench uses raw CPAL input/output. It does not provide the browser's
WebRTC echo cancellation. Use headphones when testing interruption behavior, or
the microphone may hear the agent audio and feed it back into Speech Engine.

The workbench chooses the Speech Engine TTS output format from your default
speaker rate when possible. On a typical desktop device running at 48 kHz, it
will prefer `pcm_48000`; this avoids unnecessary app-side resampling.

**Playback gain** is only a local speaker multiplier applied just before CPAL
writes samples to your output device. Keep it at `1.0` unless the agent output
clips or is too quiet.

If you reuse an existing Speech Engine, make sure the TTS output format shown in
the UI matches the engine's actual `agent_output_audio_format`. A mismatch can
sound like corrupted audio because the workbench decodes and timestamps the
frames using that selected format.

## Speech Engine config

The **Speech Engine** tab loads the selected resource when the workbench starts.
It exposes the complete Speech Engine resource configuration:

- upstream WebSocket URL and request headers
- ASR provider, quality, format, and keywords
- TTS model, voices, audio tags, normalization, pronunciation dictionaries,
  output format, and voice settings
- turn timing, eagerness, spelling behavior, speculative turns, interruption
  terms, and turn model
- client and monitoring events, file input, background sound, and source
  attribution
- recording, retention, deletion, history redaction, call limits, bursting,
  and client first-message overrides

Use **Update selected** to patch the active Speech Engine, **Create copy** to
create and select a new resource, or **Reload selected** to discard local form
edits. Reconnect after changing configuration. Restart the workbench if the TTS
output sample rate changes because the LiveKit audio source is created at
startup.

Request headers and the variable-length TTS collections use JSON editors. The
request-header value can be a string, a workspace secret locator, or a dynamic
variable locator:

```json
{
  "X-Static": "value",
  "Authorization": { "secret_id": "secret_..." },
  "X-Tenant": { "variable_name": "tenant_id" }
}
```

Supported voices, suggested audio tags, and pronunciation dictionaries are JSON
arrays matching the API request objects. Empty configurations use `{}` for
request headers and `[]` for each collection.

## Interruption behavior

The brain WebSocket follows Speech Engine's upstream interruption model: each
`user_transcript` has an `event_id`, and a newer event cancels any in-flight
Rig/Ollama response for the older event. Duplicate or older event IDs are
ignored.

The bridge keeps microphone forwarding, Speech Engine event reading, and agent
audio playback in separate tasks. This matters for barge-in: publishing agent
audio must not block microphone frames from reaching Speech Engine.

## Windows MSVC note

LiveKit's prebuilt WebRTC archive is built with the static C runtime on Windows
MSVC. This example includes `.cargo/config.toml` so `cargo run` from this
directory uses the matching static runtime. If you run it from the workspace
root, set the same flag manually:

```powershell
$env:RUSTFLAGS="-Ctarget-feature=+crt-static"
cargo run -p speech_engine_livekit
```
