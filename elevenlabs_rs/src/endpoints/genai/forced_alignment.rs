//! Forced alignment endpoints.

use super::*;
use crate::endpoints::genai::speech_to_text::TranscriptFileType;
use crate::shared::FilePart;

#[derive(Clone, Debug)]
pub struct CreateForcedAlignment {
    body: CreateForcedAlignmentBody,
}

impl CreateForcedAlignment {
    pub fn new(body: impl Into<CreateForcedAlignmentBody>) -> Self {
        Self { body: body.into() }
    }
}

#[derive(Clone, Debug)]
pub struct CreateForcedAlignmentBody {
    file: FilePart,
    text: String,
}

impl CreateForcedAlignmentBody {
    pub fn new(file: impl Into<FilePart>, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            text: text.into(),
        }
    }

    pub fn from_bytes(
        file_name: impl Into<String>,
        mime: impl Into<String>,
        bytes: impl Into<Bytes>,
        text: impl Into<String>,
    ) -> Self {
        Self::new(FilePart::bytes(file_name, mime, bytes), text)
    }
}

impl From<(String, String)> for CreateForcedAlignmentBody {
    fn from((file, text): (String, String)) -> Self {
        Self::new(file, text)
    }
}

impl crate::endpoints::sealed::Sealed for CreateForcedAlignment {}

impl ElevenLabsEndpoint for CreateForcedAlignment {
    const PATH: &'static str = "/v1/forced-alignment";

    const METHOD: Method = Method::POST;

    type ResponseBody = CreateForcedAlignmentResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        TryFrom::try_from(&self.body)
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

impl TryFrom<&CreateForcedAlignmentBody> for RequestBody {
    type Error = crate::error::Error;

    fn try_from(body: &CreateForcedAlignmentBody) -> Result<Self> {
        let inferred_mime = if body.file.mime().is_some() {
            None
        } else {
            let extension = body.file.extension()?;
            Some(TranscriptFileType::from_extension(&extension, false)?.mime_type())
        };

        Ok(RequestBody::Multipart(
            Form::new()
                .part("file", body.file.clone().into_part(inferred_mime)?)
                .text("text", body.text.clone()),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateForcedAlignmentResponse {
    pub characters: Vec<ForcedAlignmentCharacter>,
    pub words: Vec<ForcedAlignmentWord>,
    pub loss: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForcedAlignmentCharacter {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForcedAlignmentWord {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub loss: f64,
}
