#![allow(dead_code)]
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
//! use elevenlabs_rs::*;
//! use elevenlabs_rs::utils::save;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let c = ElevenLabsClient::default()?;
//!
//!     // Create a new voice preview for a whimsical, mischievous fairy character
//!     let body = CreatePreviewsBody::new(
//!         "A mischievous fairy with a playful and curious voice",
//!
//!         "Hee-hee! I bet you can't catch me! Oh, look at all the sparkles and glowing lights! /
//!         I fly faster than the wind, always tricking and teasing. /
//!         Come play with me in the forest! But beware, I love a good prank or two! /
//!         I might sprinkle pixie dust in your hair, or hide your shoes, just for fun! /
//!         What a delightful day to play tricks and spread a little mischief! /
//!         No one will ever see me coming, hee-hee!",
//!     );
//!
//!     let voice_previews = c.hit(CreatePreviews::new(body)).await?;
//!
//!     for (i, preview) in voice_previews.enumerate() {
//!         let id = preview.generated_voice_id();
//!         let sample = preview.audio_sample()?;
//!         save(&format!("fairy_sample_{}_{}.mp3", i, id), sample)?;
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use super::*;

const CREATE_PREVIEW_PATH: &str = "/v1/text-to-voice/create-previews";
const CREATE_VOICE_FROM_PREVIEW_PATH: &str = "/v1/text-to-voice/create-voice-from-preview";


/// Create previews of voices from a text
///
///
/// ## Official Documentation
/// "Generate custom voice previews based on provided voice description.
/// The response includes a list of voice previews, each containing an
/// id and a sample of the voice audio."
///
/// For further reading, check the official [create previews API reference](https://elevenlabs.io/docs/api-reference/ttv-create-previews)
///
/// ## Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use elevenlabs_rs::utils::save;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::default()?;
///     let body = CreatePreviewsBody::new(
///      "The chief orc of a fearsome army",
///
///      "Mwahahaha, marvel at my magic ye mortals! /
///       My incantation masters sound everywhere I go!
///       Mwahahaha, Mwahahaha",
///     );
///
///     let voice_previews = c.hit(CreatePreviews::new(body)).await?;
///
///     for (i, preview) in voice_previews.enumerate() {
///        let id = preview.generated_voice_id();
///        let sample = preview.audio_sample()?;
///        save(&format!("sample_{}_{}.mp3", i, id), sample)?;
///     }
///     Ok(())
/// }
/// ```
///
/// # Note
/// The text must be at least 100 characters long and at most 1000 characters long.
#[derive(Clone, Debug)]
pub struct CreatePreviews(CreatePreviewsBody);

impl CreatePreviews {
    pub fn new(body: CreatePreviewsBody) -> Self {
        Self(body)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreatePreviewsBody {
    text: String,
    voice_description: String,
}

impl CreatePreviewsBody {
    pub fn new<T: Into<String>>(voice_description: T, text: T) -> Self {
        Self {
            text: text.into(),
            voice_description: voice_description.into()
        }
    }
}

impl Endpoint for CreatePreviews {
    type ResponseBody = CreatePreviewsResponse;

    fn method(&self) -> Method {
        Method::POST
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let url = format!("{}{}", BASE_URL, CREATE_PREVIEW_PATH);
        Url::parse(&url).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreatePreviewsResponse {
    previews: Vec<VoicePreview>
}

impl Iterator for CreatePreviewsResponse {
    type Item = VoicePreview;

    fn next(&mut self) -> Option<Self::Item> {
        self.previews.pop()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoicePreview {
    audio_base_64: String,
    generated_voice_id: String,
    media_type: String,
}

impl VoicePreview {
    pub fn audio_base_64(&self) -> &str {
        &self.audio_base_64
    }

    pub fn audio_sample(&self) -> Result<Bytes> {
        let bytes = BASE64_STANDARD.decode(&self.audio_base_64)?;
        Ok(Bytes::from(bytes))
    }

    pub fn generated_voice_id(&self) -> &str {
        &self.generated_voice_id
    }

    pub fn media_type(&self) -> &str {
        &self.media_type
    }
}

/// Create a voice from a preview
///
/// ## Official Documentation
///
/// "Create a new voice from previously generated voice preview.
/// This endpoint should be called after you fetched a generated_voice_id using /v1/text-to-voice/create-previews.". i.e. `CreatePreviews`
///
/// For further reading, check the official [create voice from preview API reference](https://elevenlabs.io/docs/api-reference/ttv-create-voice-from-preview)
///
/// ## Example
/// ```no_run
/// use elevenlabs_rs::*;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let c = ElevenLabsClient::default()?;
///    let name = "Anubis";
///    let voice_description = "The chief orc of a fearsome army";
///    let some_id = "generated_voice_id";
///    let mut body = CreateVoiceFromPreviewBody::new(name, voice_description, some_id);
///    let mut labels = HashMap::new();
///    labels.insert("language".to_string(), "en".into());
///    body.with_labels(labels);
///    let resp = c.hit(CreateVoiceFromPreview::new(body)).await?;
///    println!("{:?}", resp);
///    Ok(())
/// }
/// ```
/// # Note
/// The `generated_voice_id` must be from a call to `CreatePreviews`

#[derive(Clone, Debug)]
pub struct CreateVoiceFromPreview(CreateVoiceFromPreviewBody);

impl CreateVoiceFromPreview {
    pub fn new(body: CreateVoiceFromPreviewBody) -> Self {
        Self(body)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateVoiceFromPreviewBody {
    voice_name: String,
    voice_description: String,
    generated_voice_id: String,
    labels: HashMap<String, String>,
}

impl CreateVoiceFromPreviewBody {
    pub fn new<T: Into<String>>(name: T, voice_description: T, generated_voice_id: T) -> Self {
        Self {
            voice_name: name.into(),
            voice_description: voice_description.into(),
            generated_voice_id: generated_voice_id.into(),
            labels: HashMap::new(),
        }
    }

    pub fn with_labels(&mut self, labels: HashMap<String, String>) {
        self.labels = labels;
    }
}

impl Endpoint for CreateVoiceFromPreview {
    //type ResponseBody = CreateVoiceFromPreviewResponse;
    type ResponseBody = Value;

    fn method(&self) -> Method {
        Method::POST
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.0)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }

    fn url(&self) -> Url {
        let url = format!("{}{}", BASE_URL, CREATE_VOICE_FROM_PREVIEW_PATH);
        Url::parse(&url).unwrap()
    }
}

//#[derive(Clone, Debug, Deserialize, Serialize)]
//pub struct CreateVoiceFromPreviewResponse {
//    voice_id: String,
//}





