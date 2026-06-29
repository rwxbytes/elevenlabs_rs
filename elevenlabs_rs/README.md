An unofficial Rust crate for [ElevenLabs](https://elevenlabs.io/).

MSRV: Rust 1.85.

## Features

Default features are intentionally slim for `0.7`: only native TLS is enabled.

Opt into the product areas you use:

```toml
elevenlabs_rs = { version = "0.7", features = ["genai"] }
elevenlabs_rs = { version = "0.7", features = ["convai"] }
elevenlabs_rs = { version = "0.7", features = ["admin"] }
elevenlabs_rs = { version = "0.7", features = ["genai", "ws"] }
elevenlabs_rs = { version = "0.7", features = ["genai", "playback"] }
```

TLS choices are `native-tls`, `native-tls-vendored`, `rustls`, and
`rustls-webpki-roots`. Disable default features when selecting a non-default TLS
stack.

## Text To Speech

```rust
use elevenlabs_rs::endpoints::genai::tts::{TextToSpeech, TextToSpeechBody};
use elevenlabs_rs::{DefaultVoice, ElevenLabsClient, Model, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ElevenLabsClient::from_env()?;

    let body = TextToSpeechBody::new("Hello from Rust.")
        .with_model_id(Model::ElevenMultilingualV2);

    let audio = client
        .hit(TextToSpeech::new(DefaultVoice::Brian, body))
        .await?;

    std::fs::write("speech.mp3", audio.as_ref())?;
    Ok(())
}
```

Enable `playback` if you want to use `elevenlabs_rs::utils::play`.

## Errors And Metadata

The crate returns a typed `Error`. Non-success HTTP responses are reported as
`Error::ApiError` with the status, raw body, parsed JSON error metadata, headers,
request or trace IDs, and character cost when ElevenLabs returns those values.

Use `hit_with_metadata` when response headers matter:

```rust
let response = client.hit_with_metadata(endpoint).await?;
println!("status: {}", response.status);
println!("request id: {:?}", response.request_id);
```

## Raw Endpoints

Use the raw builder when ElevenLabs ships an endpoint before this crate models it:

```rust
use elevenlabs_rs::{ElevenLabsClient, Method, Result};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ElevenLabsClient::from_env()?;

    let response: Value = client
        .raw(Method::POST, "/v1/future-endpoint")
        .json(&json!({ "text": "Hello" }))?
        .send_json()
        .await?;

    println!("{response}");
    Ok(())
}
```

Raw requests still use this crate's auth, base URL, query handling, body
encoding, typed errors, and metadata support.

## WebSockets

Enable `genai` and `ws` for realtime TTS, multi-context TTS, and realtime STT.
The `connect_*` methods return a `WebSocketSession<T>`, which is both a stream of
inbound messages and a lifecycle handle with `close`, `abort`, `is_closed`, and
`join`.

Prefer explicit `close().await` when a session is finished. Dropping a session is
safe and aborts background tasks as a fallback.

## Uploads

Upload-heavy endpoints accept `FilePart`, which can be backed by either a path or
in-memory bytes. Most upload request bodies also expose `from_bytes` constructors
for tests, services, and non-filesystem inputs.
