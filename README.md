An unofficial Rust API client for [ElevenLabs](https://elevenlabs.io/) text-to-speech software.

### API Todos

| API        | Support |
| ---------- | ------- |
| Add Voice  | ❌      |
| Edit Voice | ❌      |

## ⚙️ Requirements

- Set API key as environment variable `ELEVEN_API_KEY`

## 🗣️ Usage

```rust
use elevenlabs_rs::{Speech, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let speech = Speech::new(
        "This is the way the world ends, not with a bang but a whimper",
        "Clyde",
        "eleven_monolingual_v1",
        0,
    ).await?;
    speech.play()?;
    Ok(())
}
```

### Multilingual Model

- Generate speech using multilingual model
- Generate speech from a text file

```rust
use elevenlabs_rs::{Speech, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let speech = Speech::from_file(
        "sonnet_11.txt",
        "Glinda",
        "eleven_multilingual_v1",
        0,
    ).await?;
    speech.play()?;
    Ok(())
}
```

### Voices

- List all available voices
- Retrieve all cloned voices

```rust
use elevenlabs_rs::{get_voices, Speech, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let voices = get_voices().await?;
    let cloned_voices = voices.all_clones();

    let speech = Speech::new(
        "'I haven't the slightest idea', said the Hatter.",
        &cloned_voices[0].name.as_ref().unwrap(),
        "eleven_monolingual_v1",
        0,
    ).await?;

    speech.play()?;

     println!("Voices: {:#?}", voices);

     Ok(())
 }
```
