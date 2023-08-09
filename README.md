An unofficial Rust API client for [ElevenLabs](https://elevenlabs.io/) text-to-speech software.

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

    // None will generate a filename with the voice name and the current utc timestamp
    // e.g. Clyde_1624299999.mp3
    speech.save(None)?; // or speech.save(Some("my_file_name.mp3".to_string()))?;

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
        &cloned_voices[0].name,
        "eleven_monolingual_v1",
        0,
    ).await?;

    speech.play()?;

    println!("Voices: {:#?}", voices);

    Ok(())
 }
```

### Clone Voice

```rust
use elevenlabs_rs::{Result, Speech, VoiceCloneBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let voice_clone = VoiceCloneBuilder::new()
        .name("Ishmael") // name method is required
        .description("A very squeaky voice") // description is optional
        .label("accent", "British") // label is optional
        .label("age", "young")
        .label("gender", "male")
        .file("sample_1.mp3") // at least one file is required
        .file("sample_2.mp3")
        .build()?;

    let voice = voice_clone.add().await?;

    let speech = Speech::new(
        "I can move, I can talk, ....Am I a real boy?",
        &voice.name,
        "eleven_monolingual_v1",
        0,
    )
    .await?;

    speech.play()?;

    Ok(())
}
```

### Edit Voice

```rust
use elevenlabs_rs::{get_voices, Result};
use std::collections::HashMap;
#[tokio::main]
async fn main() -> Result<()> {
    let voices = get_voices().await?;

    let voice = voices
        .voices
        .iter()
        .find(|v| v.name == "Sabrina")
        .expect("Sabrina voice not found");

    let mut labels = HashMap::new();
    labels.insert("use case", "poetry recitation");
    labels.insert("age", "old");

    let files = vec!["new_recording.m4a", "another_new_recording.m4a"];

    let voice = voice
        .edit(
            None,         // or Some("new name")
            None,         // or Some("new description")
            Some(labels), // overwrites existing labels
            Some(files),  // appends to existing files
        )
        .await?;

    println!("Voice: {:#?}", voice);

    Ok(())
}
```
