# Speech Engine Ollama chatbot

This example runs a Speech Engine server backed by Rig and Ollama. The browser captures the system microphone with the ElevenLabs client SDK, ElevenLabs handles ASR/TTS, and this server receives transcripts over `/ws` and sends the Ollama response back.

## Prerequisites

- `ELEVENLABS_API_KEY`
- Ollama running locally
- The model pulled locally:

```powershell
ollama pull llama3.2
```

Speech Engine requires a public WebSocket URL. For local testing, expose this example with a tunnel:

```powershell
ngrok http 3001
```

Then either set an existing Speech Engine ID:

```powershell
$env:ELEVENLABS_SPEECH_ENGINE_ID="seng_..."
```

Or let the example create one by providing the public WebSocket URL:

```powershell
$env:ELEVENLABS_SPEECH_ENGINE_WS_URL="wss://your-ngrok-host.ngrok-free.app/ws"
```

## Run

```powershell
$env:ELEVENLABS_API_KEY="..."
cargo run -p speech_engine_ollama
```

Open `http://127.0.0.1:3001`, click **Start conversation**, grant microphone access, and speak.

Useful environment variables:

- `SPEECH_ENGINE_BIND` defaults to `127.0.0.1:3001`
- `OLLAMA_MODEL` defaults to `llama3.2:latest`
- `OLLAMA_API_BASE_URL` is honored by Rig's Ollama provider

If the example creates a Speech Engine, copy the printed ID into `ELEVENLABS_SPEECH_ENGINE_ID` for later runs to avoid creating duplicates.
