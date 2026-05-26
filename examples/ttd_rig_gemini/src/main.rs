use elevenlabs_rs::endpoints::genai::text_to_dialogue::*;
use elevenlabs_rs::{ElevenLabsClient, Result};
use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::gemini::{
    completion::{
        gemini_api_types::{AdditionalParameters, GenerationConfig, Schema},
        GEMINI_2_5_PRO_PREVIEW_06_05,
    },
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedDialogueInput {
    text: String,
    character: String,
}

impl TryFrom<GeneratedDialogueInput> for DialogueInput {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(turn: GeneratedDialogueInput) -> Result<Self> {
        let voice_id = match turn.character.as_str() {
            "Rachel" => "21m00Tcm4TlvDq8ikWAM",
            "Daniel" => "onwK4e9ZLuTAKqWW03F9",
            unknown => return Err(format!("Unknown character: {}", unknown).into()),
        };
        Ok(DialogueInput::new(turn.text, voice_id))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let elabs_client = ElevenLabsClient::from_env()?;
    let gemini_client = Client::from_env();

    let schema = Schema {
        r#type: "array".to_string(),
        format: None,
        description: Some("Array of dialogue turns between characters".to_string()),
        nullable: None,
        r#enum: None,
        max_items: Some(10), // Set a max limit
        min_items: Some(2),  // At least one exchange
        properties: None,
        required: None,
        items: Some(Box::new(Schema::try_from(serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The dialogue text spoken by the character"
                },
                "character": {
                    "type": "string",
                    "format": "enum",
                    "description": "Character name: either 'Rachel' or 'Daniel'",
                    "enum": ["Rachel", "Daniel"]
                }
            },
            "required": ["text", "character"]
        }))?)),
    };

    let gen_config = GenerationConfig {
        response_mime_type: Some("application/json".to_string()),
        response_schema: Some(schema),
        temperature: Some(0.7),
        ..Default::default()
    };

    let additional_params = AdditionalParameters::default().with_config(gen_config);

    let agent = gemini_client
        .agent(GEMINI_2_5_PRO_PREVIEW_06_05)
        .preamble(
            "You are a dialogue writer. Create natural conversations between characters. \
             Rachel is curious and asks questions. Daniel is knowledgeable and helpful.",
        )
        .additional_params(serde_json::to_value(additional_params)?)
        .build();

    let topic = "the future of AI and programming";
    let prompt = format!(
        "Create a short dialogue between Rachel and Daniel discussing {}. \
         Generate 4-10 exchanges with natural, conversational language.",
        topic
    );

    println!("Generating dialogue about: {}", topic);
    let response = agent.prompt(&prompt).await?;

    let gen_dialogue: Vec<GeneratedDialogueInput> = serde_json::from_str(&response)?;

    println!("\nGenerated dialogue:");
    for turn in &gen_dialogue {
        println!("{}: {}", turn.character, turn.text);
    }

    let dialogue_inputs: Result<Vec<DialogueInput>> = gen_dialogue
        .into_iter()
        .map(DialogueInput::try_from)
        .collect();

    let dialogue_inputs = dialogue_inputs?;

    println!("\nGenerating audio...");
    let body = TextToDialogueBody::new(dialogue_inputs);
    let endpoint = TextToDialogue::new(body);
    let audio_bytes = elabs_client.hit(endpoint).await?;

    let filename = "dialogue.mp3";
    std::fs::write(filename, &audio_bytes)?;
    println!("Audio saved to: {}", filename);

    Ok(())
}
