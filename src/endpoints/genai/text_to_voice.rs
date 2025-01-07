//! Voice design endpoints
//!
//! # Voice Design Guide From Official Documentation
//!
//! ## Voice Design Types
//!
//! - **Realistic Voice Design**: Create original, realistic voices by specifying attributes like age, accent/nationality, gender, tone, pitch, intonation, speed, and emotion.
//!
//!   Example prompts:
//!   - "A young Indian female with a soft, high voice. Conversational, slow, and calm."
//!   - "An old British male with a raspy, deep voice. Professional, relaxed, and assertive."
//!   - "A middle-aged Australian female with a warm, low voice. Corporate, fast, and happy."
//!
//! - **Character Voice Design**: Generate unique voices for creative characters using simpler prompts.
//!
//!   Example prompts:
//!   - "A massive evil ogre, troll."
//!   - "A sassy little squeaky mouse."
//!   - "An angry old pirate, shouting."
//!
//! Other creative character ideas include Goblin, Vampire, Elf, Troll, Werewolf, Ghost, Alien, Giant, Witch, Wizard, Zombie, Demon, Pirate, Genie, Ogre, Orc, Knight, Samurai, etc.
//!
//! ## Voice Attributes
//!
//! When designing a voice, the following attributes can be customized:
//!
//! - **Age** (High Importance): Young, Teenage, Adult, Middle-Aged, Old, etc.
//! - **Accent/Nationality** (High Importance): British, Indian, Polish, American, etc.
//! - **Gender** (High Importance): Male, Female, Gender Neutral.
//! - **Tone** (Optional): Gruff, Soft, Warm, Raspy, etc.
//! - **Pitch** (Optional): Deep, Low, High, Squeaky, etc.
//! - **Intonation** (Optional): Conversational, Professional, Corporate, Urban, Posh, etc.
//! - **Speed** (Optional): Fast, Quick, Slow, Relaxed, etc.
//! - **Emotion/Delivery** (Optional): Angry, Calm, Scared, Happy, Assertive, Whispering, Shouting, etc.
//!
//! For further reading, check the official [voice design guide documentation](https://elevenlabs.io/docs/voices/voice-lab/voice-design)
//!
//!
//! ## Example
//! ```no_run
//! use elevenlabs_rs::{ElevenLabsClient, Result};
//! use elevenlabs_rs::endpoints::genai::text_to_voice::*;
//! use elevenlabs_rs::utils::save;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let c = ElevenLabsClient::from_env()?;
//!
//!     let text = " Hee-hee! I bet you can't catch me! Oh, look at all the sparkles and glowing lights! /
//!         I fly faster than the wind, always tricking and teasing. /
//!         Come play with me in the forest! But beware, I love a good prank or two! /
//!         I might sprinkle pixie dust in your hair, or hide your shoes, just for fun! /
//!         What a delightful day to play tricks and spread a little mischief! /
//!         No one will ever see me coming, hee-hee!";
//!
//!     let body = TextToVoiceBody::new("a mischievous fairy with a playful and curious voice")
//!         .with_text(text);
//!
//!     let voice_previews = c.hit(TextToVoice::new(body)).await?;
//!
//!     for (i, preview) in voice_previews.into_iter().enumerate() {
//!         let id = &preview.generated_voice_id;
//!         let sample = preview.audio_sample()?;
//!         save(&format!("fairy_sample_{}_{}.mp3", i, id), sample)?;
//!     }
//!
//!     Ok(())
//! }
//! ```

use super::*;
use std::collections::HashMap;

/// Generate voices from a single text prompt.
///
/// ## Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::text_to_voice::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///
///     let text = "Mwahahaha, marvel at my magic ye mortals! /
///         My incantation masters sound everywhere I go! /
///         Mwahahaha, Mwahahaha";
///
///     let body = TextToVoiceBody::new("The chief orc of a fearsome army").with_text(text);
///
///     let voice_previews = c.hit(TextToVoice::new(body)).await?;
///
///     for (i, preview) in voice_previews.into_iter().enumerate() {
///        let id = &preview.generated_voice_id;
///        let sample = preview.audio_sample()?;
///        save(&format!("sample_{}_{}.mp3", i, id), sample)?;
///     }
///     Ok(())
/// }
/// ```
/// # Note
/// The text must be at least 100 characters long and at most 1000 characters long.
///
/// See [Text To Voice API reference](https://elevenlabs.io/docs/api-reference/text-to-voice/create-previews)
#[derive(Clone, Debug)]
pub struct TextToVoice {
    body: TextToVoiceBody,
    query: Option<TextToVoiceQuery>,
}

impl TextToVoice {
    pub fn new(body: TextToVoiceBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToVoiceQuery) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextToVoiceBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    voice_description: String,
    auto_generate_text: bool,
}

impl TextToVoiceBody {
    pub fn new(voice_description: impl Into<String>) -> Self {
        Self {
            text: None,
            voice_description: voice_description.into(),
            auto_generate_text: false,
        }
    }

    pub fn with_auto_generated_text(mut self) -> Self {
        self.auto_generate_text = true;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct TextToVoiceQuery {
    params: QueryValues,
}

impl TextToVoiceQuery {
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format".into(), output_format.to_string()));
        self
    }
}

impl ElevenLabsEndpoint for TextToVoice {
    const PATH: &'static str = "/v1/text-to-voice/create-previews";

    const METHOD: Method = Method::POST;

    type ResponseBody = TextToVoiceResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextToVoiceResponse {
    pub previews: Vec<VoicePreview>,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoicePreview {
    pub audio_base_64: String,
    pub generated_voice_id: String,
    pub media_type: String,
    pub duration_secs: f32,
}

impl VoicePreview {
    pub fn audio_sample(&self) -> Result<Bytes> {
        let bytes = BASE64_STANDARD.decode(&self.audio_base_64)?;
        Ok(Bytes::from(bytes))
    }
}

impl IntoIterator for TextToVoiceResponse {
    type Item = VoicePreview;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.previews.into_iter()
    }
}

/// Add a generated voice to the voice library.
///
/// ## Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::genai::text_to_voice::*;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::from_env()?;
///    let name = "Anubis";
///    let voice_description = "The chief orc of a fearsome army";
///    let some_id = "generated_voice_id";
///    let mut body = SaveVoiceFromPreviewBody::new(name, voice_description, some_id);
///    let mut labels = HashMap::new();
///    labels.insert("language".to_string(), "en".into());
///    body.with_labels(labels);
///    let resp = c.hit(SaveVoiceFromPreview::new(body)).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// # Note
/// The `generated_voice_id` must be from a call to `CreatePreviews`
///
/// See [Save Voice from Preview API reference](https://elevenlabs.io/docs/api-reference/text-to-voice/create-voice-from-preview)
#[derive(Clone, Debug)]
pub struct SaveVoiceFromPreview {
    body: SaveVoiceFromPreviewBody,
}

impl SaveVoiceFromPreview {
    pub fn new(body: SaveVoiceFromPreviewBody) -> Self {
        Self { body }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SaveVoiceFromPreviewBody {
    voice_name: String,
    voice_description: String,
    generated_voice_id: String,
    labels: HashMap<String, String>,
    played_not_selected_voice_ids: Option<Vec<String>>,
}

impl SaveVoiceFromPreviewBody {
    pub fn new<T: Into<String>>(name: T, voice_description: T, generated_voice_id: T) -> Self {
        Self {
            voice_name: name.into(),
            voice_description: voice_description.into(),
            generated_voice_id: generated_voice_id.into(),
            labels: HashMap::new(),
            played_not_selected_voice_ids: None,
        }
    }

    pub fn with_labels(&mut self, labels: HashMap<String, String>) {
        self.labels = labels;
    }
}

impl ElevenLabsEndpoint for SaveVoiceFromPreview {
    const PATH: &'static str = "/v1/text-to-voice/create-voice-from-preview";
    const METHOD: Method = Method::POST;

    //type ResponseBody = CreateVoiceFromPreviewResponse;
    type ResponseBody = Value;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

// TODO: Implement this response body
//#[derive(Clone, Debug, Deserialize, Serialize)]
//pub struct CreateVoiceFromPreviewResponse {
//    voice_id: String,
//}
