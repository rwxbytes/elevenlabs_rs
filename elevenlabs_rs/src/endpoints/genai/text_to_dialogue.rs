//! The text-to-dialogue endpoint
use super::*;
use crate::shared::{query_params::OutputFormat, DictionaryLocator, VoiceSettings};

/// Converts a list of text and voice ID pairs into speech (dialogue) and returns audio.
///
/// See [Text-to-Dialogue API reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue/convert)
#[derive(Clone, Debug)]
pub struct TextToDialogue {
    body: TextToDialogueBody,
    query: Option<TextToDialogueQuery>,
}

impl TextToDialogue {
    pub fn new(body: TextToDialogueBody) -> Self {
        Self { body, query: None }
    }

    pub fn with_query(mut self, query: TextToDialogueQuery) -> Self {
        self.query = Some(query);
        self
    }
}

impl ElevenLabsEndpoint for TextToDialogue {
    const PATH: &'static str = "/v1/text-to-dialogue";

    const METHOD: Method = Method::POST;

    type ResponseBody = Bytes;

    fn query_params(&self) -> Option<QueryValues> {
        self.query.as_ref().map(|q| q.params.clone())
    }

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.bytes().await?)
    }
}

/// Single dialogue turn input consisting of text and a voice ID
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogueInput {
    pub text: String,
    pub voice_id: String,
}

impl DialogueInput {
    pub fn new(text: impl Into<String>, voice_id: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            voice_id: voice_id.into(),
        }
    }
}

/// Request body for Text-to-Dialogue API
#[derive(Clone, Debug, Serialize, Default)]
pub struct TextToDialogueBody {
    pub inputs: Vec<DialogueInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<VoiceSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronunciation_dictionary_locators: Option<Vec<DictionaryLocator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

impl TextToDialogueBody {
    pub fn new(inputs: Vec<DialogueInput>) -> Self {
        Self {
            inputs,
            ..Default::default()
        }
    }

    pub fn with_model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn with_settings(mut self, settings: VoiceSettings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn with_pronunciation_dictionary_locators(
        mut self,
        locators: Vec<DictionaryLocator>,
    ) -> Self {
        self.pronunciation_dictionary_locators = Some(locators);
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

impl TryFrom<&TextToDialogueBody> for RequestBody {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: &TextToDialogueBody) -> Result<Self> {
        Ok(RequestBody::Json(serde_json::to_value(value)?))
    }
}


/// Query parameters for Text-to-Dialogue API
#[derive(Clone, Debug, Default)]
pub struct TextToDialogueQuery {
    pub params: QueryValues,
}

impl TextToDialogueQuery {
    pub fn with_output_format(mut self, output_format: OutputFormat) -> Self {
        self.params
            .push(("output_format", output_format.to_string()));
        self
    }
}
