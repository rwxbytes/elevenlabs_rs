//! Conversational AI outbound telephony endpoints (Exotel, SIP trunk, WhatsApp).
//!
//! These endpoints place an outbound conversational-AI call (or send a WhatsApp
//! message) through the corresponding provider.
//!
//! See the [Conversational AI API reference](https://elevenlabs.io/docs/conversational-ai/api-reference).

use super::*;
use crate::endpoints::convai::batch_calling::TelephonyCallConfig;
use crate::endpoints::convai::conversations::ConversationInitiationClientData;

/// Shared body for provider outbound calls that dial a phone number (Exotel, SIP trunk).
#[derive(Clone, Debug, Serialize)]
pub struct OutboundCallBody {
    agent_id: String,
    agent_phone_number_id: String,
    to_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    telephony_call_config: Option<TelephonyCallConfig>,
}

impl OutboundCallBody {
    pub fn new(
        agent_id: impl Into<String>,
        agent_phone_number_id: impl Into<String>,
        to_number: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            agent_phone_number_id: agent_phone_number_id.into(),
            to_number: to_number.into(),
            conversation_initiation_client_data: None,
            telephony_call_config: None,
        }
    }

    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }

    pub fn with_telephony_call_config(
        mut self,
        telephony_call_config: TelephonyCallConfig,
    ) -> Self {
        self.telephony_call_config = Some(telephony_call_config);
        self
    }
}

// =============================================================================
// POST /v1/convai/exotel/outbound-call
// =============================================================================

/// Place an outbound call via Exotel.
///
/// See [Exotel Outbound Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/exotel/outbound-call)
#[derive(Clone, Debug)]
pub struct ExotelOutboundCall {
    body: OutboundCallBody,
}

impl ExotelOutboundCall {
    pub fn new(body: OutboundCallBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for ExotelOutboundCall {}

impl ElevenLabsEndpoint for ExotelOutboundCall {
    const PATH: &'static str = "/v1/convai/exotel/outbound-call";

    const METHOD: Method = Method::POST;

    type ResponseBody = ExotelOutboundCallResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ExotelOutboundCall`].
#[derive(Clone, Debug, Deserialize)]
pub struct ExotelOutboundCallResponse {
    pub success: bool,
    pub message: String,
    pub conversation_id: Option<String>,
    #[serde(rename = "callSid")]
    pub call_sid: Option<String>,
}

// =============================================================================
// POST /v1/convai/sip-trunk/outbound-call
// =============================================================================

/// Place an outbound call via a SIP trunk.
///
/// See [SIP Trunk Outbound Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/sip-trunk/outbound-call)
#[derive(Clone, Debug)]
pub struct SipTrunkOutboundCall {
    body: OutboundCallBody,
}

impl SipTrunkOutboundCall {
    pub fn new(body: OutboundCallBody) -> Self {
        Self { body }
    }
}

impl crate::endpoints::sealed::Sealed for SipTrunkOutboundCall {}

impl ElevenLabsEndpoint for SipTrunkOutboundCall {
    const PATH: &'static str = "/v1/convai/sip-trunk/outbound-call";

    const METHOD: Method = Method::POST;

    type ResponseBody = SipTrunkOutboundCallResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`SipTrunkOutboundCall`].
#[derive(Clone, Debug, Deserialize)]
pub struct SipTrunkOutboundCallResponse {
    pub success: bool,
    pub message: String,
    pub conversation_id: Option<String>,
    pub sip_call_id: Option<String>,
}

// =============================================================================
// POST /v1/convai/whatsapp/outbound-call
// =============================================================================

/// Place an outbound call via WhatsApp.
///
/// See [WhatsApp Outbound Call API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/outbound-call)
#[derive(Clone, Debug)]
pub struct WhatsAppOutboundCall {
    body: WhatsAppOutboundCallBody,
}

impl WhatsAppOutboundCall {
    pub fn new(body: WhatsAppOutboundCallBody) -> Self {
        Self { body }
    }
}

/// Body for [`WhatsAppOutboundCall`].
#[derive(Clone, Debug, Serialize)]
pub struct WhatsAppOutboundCallBody {
    agent_id: String,
    whatsapp_phone_number_id: String,
    whatsapp_user_id: String,
    whatsapp_call_permission_request_template_name: String,
    whatsapp_call_permission_request_template_language_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl WhatsAppOutboundCallBody {
    pub fn new(
        agent_id: impl Into<String>,
        whatsapp_phone_number_id: impl Into<String>,
        whatsapp_user_id: impl Into<String>,
        template_name: impl Into<String>,
        template_language_code: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            whatsapp_phone_number_id: whatsapp_phone_number_id.into(),
            whatsapp_user_id: whatsapp_user_id.into(),
            whatsapp_call_permission_request_template_name: template_name.into(),
            whatsapp_call_permission_request_template_language_code: template_language_code.into(),
            conversation_initiation_client_data: None,
        }
    }

    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }
}

impl crate::endpoints::sealed::Sealed for WhatsAppOutboundCall {}

impl ElevenLabsEndpoint for WhatsAppOutboundCall {
    const PATH: &'static str = "/v1/convai/whatsapp/outbound-call";

    const METHOD: Method = Method::POST;

    type ResponseBody = WhatsAppOutboundCallResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`WhatsAppOutboundCall`].
#[derive(Clone, Debug, Deserialize)]
pub struct WhatsAppOutboundCallResponse {
    pub success: bool,
    pub message: String,
    pub conversation_id: Option<String>,
}

// =============================================================================
// POST /v1/convai/whatsapp/outbound-message
// =============================================================================

/// Send an outbound message via WhatsApp.
///
/// See [WhatsApp Outbound Message API reference](https://elevenlabs.io/docs/conversational-ai/api-reference/whatsapp/outbound-message)
#[derive(Clone, Debug)]
pub struct WhatsAppOutboundMessage {
    body: WhatsAppOutboundMessageBody,
}

impl WhatsAppOutboundMessage {
    pub fn new(body: WhatsAppOutboundMessageBody) -> Self {
        Self { body }
    }
}

/// Body for [`WhatsAppOutboundMessage`].
#[derive(Clone, Debug, Serialize)]
pub struct WhatsAppOutboundMessageBody {
    agent_id: String,
    whatsapp_phone_number_id: String,
    whatsapp_user_id: String,
    template_name: String,
    template_language_code: String,
    template_params: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversation_initiation_client_data: Option<ConversationInitiationClientData>,
}

impl WhatsAppOutboundMessageBody {
    pub fn new(
        agent_id: impl Into<String>,
        whatsapp_phone_number_id: impl Into<String>,
        whatsapp_user_id: impl Into<String>,
        template_name: impl Into<String>,
        template_language_code: impl Into<String>,
        template_params: impl IntoIterator<Item = Value>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            whatsapp_phone_number_id: whatsapp_phone_number_id.into(),
            whatsapp_user_id: whatsapp_user_id.into(),
            template_name: template_name.into(),
            template_language_code: template_language_code.into(),
            template_params: template_params.into_iter().collect(),
            conversation_initiation_client_data: None,
        }
    }

    pub fn with_conversation_initiation_client_data(
        mut self,
        data: ConversationInitiationClientData,
    ) -> Self {
        self.conversation_initiation_client_data = Some(data);
        self
    }
}

impl crate::endpoints::sealed::Sealed for WhatsAppOutboundMessage {}

impl ElevenLabsEndpoint for WhatsAppOutboundMessage {
    const PATH: &'static str = "/v1/convai/whatsapp/outbound-message";

    const METHOD: Method = Method::POST;

    type ResponseBody = WhatsAppOutboundMessageResponse;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`WhatsAppOutboundMessage`].
#[derive(Clone, Debug, Deserialize)]
pub struct WhatsAppOutboundMessageResponse {
    pub conversation_id: String,
}
