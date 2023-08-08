use crate::{api::ClientBuilder, error::Error, prelude::*};
use comparable::*;
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
};

const BASE_PATH: &str = "/voices";
const SETTINGS_PATH: &str = "/settings";
const DEFAULT_SETTINGS_PATH: &str = "/voices/settings/default";
const EDIT_PATH: &str = "/edit";
const ADD_PATH: &str = "/add";

#[cfg(test)]
mod test {
    use super::*;

    // Premade Rachel's voice id
    const RACHEL_VOICE_ID: &str = "21m00Tcm4TlvDq8ikWAM";

    #[tokio::test]
    #[ignore]
    async fn get_voice_is_returing_a_voice_when_given_valid_voice_id() {
        let want = Voice {
            voice_id: RACHEL_VOICE_ID.to_string(),
            name: "Rachel".to_string(),
            samples: None,
            category: Some("premade".to_string()),
            // TODO: Fix: This is not the same as the one in the API.
            // "lables": {"accent": "american", "description": "calm", "age": "young", "gender": "female", "use_case": "narration"}
            labels: Some(Labels {
                additional_prop1: None,
                additional_prop2: None,
                additional_prop3: None,
            }),
            description: None,
            preview_url: Some("https://storage.googleapis.com/eleven-public-prod/premade/voices/21m00Tcm4TlvDq8ikWAM/6edb9076-c3e4-420c-b6ab-11d43fe341c8.mp3".to_string()),
            settings: None,
        };

        let got = get_voice(RACHEL_VOICE_ID, false).await.unwrap();
        let identity = want.comparison(&got);
        if !identity.is_unchanged() {
            panic!("identity: {:#?}", identity);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn get_voice_is_errring_when_given_invalid_voice_id() {
        let voice_id = "bogus_voice_id";
        let got = get_voice(voice_id, false).await;
        assert!(got.is_err());
    }
    #[test]
    fn voice_clone_builders_build_is_errring_when_its_name_is_none() {
        let vcb = VoiceCloneBuilder::new();
        let _voice_clone = vcb.file("file_name").build();
        assert!(_voice_clone.is_err());
    }
    #[test]
    fn voice_clone_builders_build_is_errring_when_its_files_is_empty() {
        let vcb = VoiceCloneBuilder::new();
        let _voice_clone = vcb.name("test").build();
        assert!(_voice_clone.is_err());
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Comparable)]
pub struct Voices {
    pub voices: Vec<Voice>,
}

impl Voices {
    pub fn all_clones(&self) -> Vec<&Voice> {
        self.voices
            .iter()
            .filter(|v| v.category == Some("cloned".to_string()))
            .collect::<Vec<&Voice>>()
    }
    pub fn by_name(&self, name: &str) -> Result<&Voice> {
        self.voices
            .iter()
            .find(|v| v.name == name.to_string())
            .ok_or(Box::new(Error::VoiceNotFound))
    }
    pub async fn get_voice_with_settings(&self, voice_name: &str) -> Result<Voice> {
        let voice = self.by_name(voice_name)?;
        let voice = get_voice(&voice.voice_id, true).await?;
        Ok(voice)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Comparable)]
pub struct Voice {
    pub voice_id: String,
    pub name: String,
    pub samples: Option<Vec<VoiceSample>>,
    pub category: Option<String>,
    pub labels: Option<Labels>,
    pub description: Option<String>,
    pub preview_url: Option<String>,
    pub settings: Option<VoiceSettings>,
}

impl Voice {
    //pub async fn add(
    //    name: &str,
    //    description: Option<String>,
    //    labels: Option<HashMap<String, String>>,
    //    files: Vec<String>,
    //) -> Result<Voice> {
    //    let vc = VoiceClone::new(name, description);
    //}
    pub async fn with_settings(voice_name: &str) -> Result<Self> {
        let voices = get_voices().await?;
        let voice = voices.by_name(voice_name)?;
        let voice = get_voice(&voice.voice_id, true).await?;
        Ok(voice)
    }
}

#[derive(Serialize, Deserialize, Comparable, Debug, Clone, PartialEq)]
pub struct VoiceSample {
    pub sample_id: String,
    file_name: String,
    mime_type: String,
    size_bytes: Option<i64>,
    hash: String,
}

#[derive(Debug)]
pub struct VoiceClone {
    name: String,
    description: String,
    files: Vec<String>,
    labels: HashMap<String, String>,
}

#[derive(Default)]
pub struct VoiceCloneBuilder {
    name: Option<String>,
    description: Option<String>,
    files: Vec<String>,
    labels: Option<HashMap<String, String>>,
}

impl VoiceCloneBuilder {
    /// Create a new VoiceCloneBuilder
    ///
    /// # Example
    ///
    /// ```no_run
    ///use elevenlabs_rs::{Result, Speech, VoiceCloneBuilder};
    ///
    ///#[tokio::main]
    ///async fn main() -> Result<()> {
    ///    let voice_clone = VoiceCloneBuilder::new()
    ///        .name("Ronald")
    ///        .description("A cockney underworld boss")
    ///        .label("accent", "British")
    ///        .label("age", "middle aged")
    ///        .file("sample_1.mp3")
    ///        .file("sample_2.mp3")
    ///        .file("sample_3.mp3")
    ///        .build()?;
    ///
    ///    let voice = voice_clone.add().await?;
    ///
    ///    let speech = Speech::new(
    ///        "Hence the expression, 'As greedy as a pig'.",
    ///        &voice.name,
    ///        "eleven_multilingual_v1",
    ///        0,
    ///    )
    ///    .await?;
    ///
    ///    speech.play()?;
    ///
    ///    Ok(())
    ///}
    /// ```
    pub fn new() -> Self {
        VoiceCloneBuilder::default()
    }
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    pub fn file(mut self, file_name: &str) -> Self {
        self.files.push(file_name.to_string());
        self
    }
    pub fn label(mut self, key: &str, value: &str) -> Self {
        self.labels
            .get_or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
        self
    }
    pub fn build(self) -> Result<VoiceClone> {
        let name = self.name.ok_or(Box::new(Error::VoiceCloneBuilderError(
            "name must be set".to_string(),
        )))?;
        if self.files.is_empty() {
            return Err(Box::new(Error::VoiceCloneBuilderError(
                "At least one file must be given".to_string(),
            )));
        }
        let files = self.files;
        let description = self.description;
        let labels = self.labels;
        Ok(VoiceClone {
            name,
            description: description.unwrap_or_default(),
            files,
            labels: labels.unwrap_or_default(),
        })
    }
}

impl VoiceClone {
    pub async fn add(&self) -> Result<Voice> {
        let boundary = format!(
            "-----------------------------{}",
            rand::thread_rng().gen::<u64>()
        );

        let data = self.to_multipart_form_data(&boundary)?;

        let c = ClientBuilder::new()?
            .method(POST)?
            .path(format!("{}{}", BASE_PATH, ADD_PATH))?
            .header(
                CONTENT_TYPE,
                &format!("{}{}", MULTIPART_FORM_DATA_BOUNDARY, boundary),
            )?
            .header(ACCEPT, APPLICATION_JSON)?
            .build()?;

        let resp = c.send_request(Full::<Bytes>::new(data.into())).await?;

        let json = serde_json::from_slice::<serde_json::Value>(&resp)?;

        if let Some(voice_id) = json["voice_id"].as_str() {
            return Ok(get_voice(voice_id, true).await?);
        } else {
            return Err(Box::new(Error::ClientSendRequestError(json)));
        }
    }

    fn to_multipart_form_data(&self, boundary: &str) -> io::Result<Vec<u8>> {
        let mut data = Vec::new();

        write!(data, "--{}\r\n", boundary)?;
        write!(
            data,
            "Content-Disposition: form-data; name=\"name\"\r\n\r\n{}\r\n",
            self.name
        )?;

        if !self.description.is_empty() {
            write!(data, "--{}\r\n", boundary)?;
            write!(
                data,
                "Content-Disposition: form-data; name=\"description\"\r\n\r\n{}\r\n",
                &self.description
            )?;
        }

        for file_path in &self.files {
            write!(data, "--{}\r\n", boundary)?;
            write!(
                data,
                "Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",
                file_path
            )?;
            write!(data, "Content-Type: audio/mpeg\r\n\r\n")?;

            let mut f = File::open(file_path)?;
            f.read_to_end(&mut data)?;

            write!(data, "\r\n")?;
        }

        write!(data, "--{}\r\n", boundary)?;
        write!(
            data,
            "Content-Disposition: form-data; name=\"labels\"\r\n\r\n{}\r\n",
            serde_json::to_string(&self.labels).unwrap()
        )?;

        write!(data, "--{}--\r\n", boundary)?;

        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Comparable, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Labels {
    additional_prop1: Option<String>,
    additional_prop2: Option<String>,
    additional_prop3: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Comparable)]
pub struct VoiceSettings {
    pub similarity_boost: f64,
    pub stability: f64,
    pub style: f64,
    pub use_speaker_boost: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            similarity_boost: 0.75,
            stability: 0.5,
            style: 0.0,
            use_speaker_boost: true,
        }
    }
}

pub async fn get_voices() -> Result<Voices> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(GET)?
        .path(BASE_PATH)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let voices: Voices = serde_json::from_slice(&resp)?;
    Ok(voices)
}

pub async fn get_default_settings() -> Result<VoiceSettings> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(GET)?
        .path(DEFAULT_SETTINGS_PATH)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let voices_settings = serde_json::from_slice::<VoiceSettings>(&resp)?;
    Ok(voices_settings)
}

/// Get the voice settings for a specific voice
///
/// # Example
///```
///  use elevenlabs_rs::api::voice::{get_voice_settings, get_voices};
///  use elevenlabs_rs::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let v = get_voices().await?;
///     let cloned_voices = v.all_clones();
///     let settings = get_voice_settings(&cloned_voices[0].voice_id).await?;
///     println!("Settings: {:#?}", settings);
///     Ok(())
/// }
///
/// // prints:
///
/// // Settings: VoiceSettings {
/// //     similarity_boost: 0.125,
/// //     stability: 0.325,
/// //     style: 0.0,
/// //     use_speaker_boost: false,
/// // }
/// ```
pub async fn get_voice_settings(voice_id: &str) -> Result<VoiceSettings> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(GET)?
        .path(format!("{}/{}{}", BASE_PATH, voice_id, SETTINGS_PATH))?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let voices_settings = serde_json::from_slice::<VoiceSettings>(&resp)?;
    Ok(voices_settings)
}

pub async fn get_voice(voice_id: &str, with_settings: bool) -> Result<Voice> {
    let path = if with_settings {
        format!("{}/{}?with_settings=true", BASE_PATH, voice_id)
    } else {
        format!("{}/{}", BASE_PATH, voice_id)
    };
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(GET)?
        .path(path)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let voice = serde_json::from_slice::<Voice>(&resp)?;
    Ok(voice)
}

pub async fn delete_voice(voice_id: &str) -> Result<()> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(DELETE)?
        .path(format!("{}/{}", BASE_PATH, voice_id))?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let _resp = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(())
}

/// Edit the voice settings for a specific voice
/// # Example
/// ```
/// use elevenlabs_rs::api::voice::*;
/// use elevenlabs_rs::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    let voice = Voice::with_settings("Adam").await?;
///
///    let settings = VoiceSettings {
///         similarity_boost: 0.125,
///         stability: 0.325,
///         style: 0.0,
///         use_speaker_boost: true,
///    };
///
///    edit_voice_settings(&voice.voice_id, settings).await?;
///
///    Ok(())
/// }
/// ```
pub async fn edit_voice_settings(voice_id: &str, settings: VoiceSettings) -> Result<()> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .method(POST)?
        .path(format!(
            "{}/{}{}{}",
            BASE_PATH, voice_id, SETTINGS_PATH, EDIT_PATH
        ))?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let _resp = c
        .send_request(Full::<Bytes>::new(serde_json::to_string(&settings)?.into()))
        .await?;
    Ok(())
}

pub async fn add_voice() -> Result<()> {
    todo!("Add voice endpoint")
}

pub async fn edit_voice() -> Result<()> {
    todo!("Edit voice endpoint");
}
