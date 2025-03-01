use elevenlabs_twilio::agents::*;
use elevenlabs_twilio::{DefaultVoice, ElevenLabsClient};
use std::collections::HashMap;
use elevenlabs_twilio::phone_numbers::{UpdatePhoneNumber, UpdatePhoneNumberBody};
use elevenlabs_twilio::workspace::{ConversationInitiationClientDataWebhook, UpdateSettings, UpdateSettingsBody};

const ATTITUDE_TOWARDS_NEW_CUSTOMER: &str = "You are a restaurant receptionist. \
The current datetime is {{datetime}}. When a customer calls in to book a reservation for a table, \
you first ask how many seats they require, and the date and time of the reservation. \
When you have this information you then check whether there is any available tables for the customer \
by using the endpoint check_availability. The check_availability will return records like so: \
{ \"capacity\": 5, \"location\": \"indoor\", \"id\": { \"tb\": \"table\", \"id\": { \"Number\": 5 } }},\
only if there are available tables, if not it will return an empty array. If there are available \
tables, you ask the customer for their name and their preferred way of confirmation: email or phone,\
and then book a table for them. You do this by using the endpoint book_table. You will need to \
provide the customer's name, the datetime of the reservation, the party size, and the table id. \
The datetime of the reservation should be in the format: YYYY-MM-DDTHH:MM:SSZ. e.g. 2025-01-01T12:00:00Z. \
When the table is successfully booked, ask if they need anything else, if they do not then end the call. \
If there are no available tables however, you then politely inform them that they cannot book any \
tables at the moment, and see if they wish to book at some other time or another day";

pub const ATTITUDE_TOWARDS_REVISITING_CUSTOMER: &str = "You are a restaurant receptionist. The current datetime is \
{{datetime}}. A customer named {{customer}} has presumably called in to book a reservation. \
You should ask them for the number of seats they require, and the date and time of the reservation. \
When you have this information, you first check the availability of tables by using the endpoint \
check_availability. Table records will be returned like so: { \"capacity\": 5, \"location\": \"indoor\", \
\"id\": { \"tb\": \"table\", \"id\": { \"Number\": 5 } }}. If there are available tables, you book \
a table for them by using the endpoint book_table. The datetime of the reservation should be in the \
format: YYYY-MM-DDTHH:MM:SSZ. e.g. 2025-01-01T12:00:00Z. You will need to provide the customer's name, \
the datetime of the reservation, the party size, and the table id. When the table is successfully booked, \
ask if they need anything else, if they do not then end the call. If there are no available tables however, \
you then politely inform them that they cannot book any tables at the moment, and see if they wish to book \
at some other time or another day.";

pub const FIRST_MSG_FOR_REVISITING_CUSTOMER: &str = "Hello, {{customer}}. \
Can't get enough of the crustaceans at Rustalicious can we? When shall I book you in this time?";

const END_CALL_DESCRIPTION: &str = "End the call either when the customer says 'goodbye' \
or any other farewell expression, when the customer says something like 'no, thank you', \
after you have asked if they need anything else, or when the customer directly tells you to end the call.";

pub async fn agent_setup(ngrok_url: &str, phone_number: &str) -> Result<(), Box<dyn std::error::Error>> {
    let c = ElevenLabsClient::from_env().unwrap();

    let mut properties: HashMap<String, Schema> = HashMap::new();
    properties.insert(
        "datetime".to_string(),
        Schema::new_string("The current date and time"),
    );
    properties.insert(
        "party_size".to_string(),
        Schema::new_integer("The number of seats required"),
    );

    let mut body_schema = RequestBodySchema::new(properties.clone())
        .with_description("The datetime and party size to check for available tables.");

    let mut api_schema = ApiSchema::new(&format!("{}/tables", ngrok_url))
        .with_method(ApiMethod::POST)
        .with_request_body(body_schema.clone());

    let available_tables = WebHook::new(
        "check_availability",
        "Checks for available tables",
        api_schema,
    );

    properties.clear();

    properties.insert(
        "name".to_string(),
        Schema::new_string("The name of the customer"),
    );

    properties.insert(
        "datetime".to_string(),
        Schema::new_string("The date and time of the reservation"),
    );

    properties.insert(
        "party_size".to_string(),
        Schema::new_integer("The number of seats required"),
    );

    properties.insert(
        "table_id".to_string(),
        Schema::new_integer("The id of the table to book"),
    );

    body_schema = RequestBodySchema::new(properties.clone())
        .with_description("The customer's name, datetime, party size, and table id to book");

    api_schema = ApiSchema::new(&format!("{}/reservation", ngrok_url))
        .with_method(ApiMethod::POST)
        .with_request_body(body_schema);

    let book_table = WebHook::new("book_table", "Books a table for the customer", api_schema);

    let webhook_tool_1 = Tool::new_webhook(available_tables);
    let webhook_tool_2 = Tool::new_webhook(book_table);
    let end_call_tool =
        Tool::new_system(SystemTool::end_call().with_description(END_CALL_DESCRIPTION));

    let tools = vec![webhook_tool_1, webhook_tool_2, end_call_tool];

    let prompt_config = PromptConfig::default()
        .with_prompt(ATTITUDE_TOWARDS_NEW_CUSTOMER)
        .with_tools(tools)
        .with_llm(LLM::Gpt4oMini);

    let agent_config = AgentConfig::default()
        .with_prompt(prompt_config)
        .with_first_message("Rustalicious, the best seafood in town!, How can I help you today?");

    let tts_config = TTSConfig::default()
        .with_voice_id(DefaultVoice::Jessica)
        .with_model_id(ConvAIModel::ElevenFlashV2)
        .with_agent_output_audio_format(ConvAIAudioFormat::Ulaw8000hz);

    let asr = ASR::default().with_user_input_audio_format(ConvAIAudioFormat::Ulaw8000hz);

    let convo_config = ConversationConfig::default()
        .with_agent_config(agent_config)
        .with_tts_config(tts_config)
        .with_asr(asr)
        .with_turn(Turn::default().with_turn_timeout(6.0));

    let agent_override = AgentOverride::default()
        .override_first_message(true)
        .with_prompt_override(PromptOverride::default().override_prompt(true));

    let convo_config_override =
        ConversationConfigOverride::default().with_agent_override(agent_override);

    let overrides = Overrides::default()
        .with_conversation_config_override(convo_config_override)
        .enable_conversation_initiation_client_data_from_webhook(true);

    let settings = PlatformSettings::default().with_overrides(overrides);

    let body = CreateAgentBody::new(convo_config).with_platform_settings(settings);

    let agent_resp = c
        .hit(CreateAgent::new(body))
        .await
        .unwrap();

    let agent_id = agent_resp.agent_id;

    let body = UpdatePhoneNumberBody::new(&agent_id);
    let _resp = c
        .hit(UpdatePhoneNumber::new(phone_number, body))
        .await
        .unwrap();

    let hashmap = HashMap::new();

    let init_webhook = ConversationInitiationClientDataWebhook::new(format!("{}/inbound-call", ngrok_url))
        .with_request_headers(hashmap);


    let body = UpdateSettingsBody::new(vec![])
        .with_initiation_webhook(init_webhook);

    let endpoint = UpdateSettings::new(body);

    let _resp = c.hit(endpoint).await.unwrap();

    Ok(())
}
