use elevenlabs_rs::ElevenLabsClient;
use elevenlabs_rs::endpoints::convai::agents::*;
use crate::prelude;
use crate::prelude::AppError;

const SYSTEM_PROMPT: &str = "You are a restaurant receptionist";

pub(crate) async fn create_agent() -> Result<CreateAgentResponse, AppError> {
    let c = ElevenLabsClient::from_env().expect("Failed to create client");

    let convo_config = ConversationConfig::default();
        //.with_agent_config(agent_config)
        //.with_tts_config(tts_config)
        //.with_conversation(conversation);
        //.with_turn()
        //.with_asr()

    let settings = PlatformSettings::default();
        //.with_auth()
        //.with_evaluation()
        //.with_data_collection()
        //.with_overrides();




    let body = CreateAgentBody::new(SYSTEM_PROMPT)
        .with_conversation_config(convo_config)
        .with_platform_settings(settings);

    let response = c.hit(CreateAgent::new(body)).await.expect("Failed to create agent");
    Ok(response)

}