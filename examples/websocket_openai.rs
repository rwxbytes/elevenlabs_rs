use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use async_stream::stream;
use elevenlabs_rs::endpoints::tts::ws::*;
use elevenlabs_rs::utils::stream_audio;
use elevenlabs_rs::*;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let elevenlabs_client = ElevenLabsClient::default()?;
    let model_id = Model::ElevenTurboV2;
    let voice_id = PreMadeVoiceID::Rachel;

    let text_stream = stream! {

        let openai_client = Client::new();
        let chat = openai_client.chat();
        let chat_req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .messages(vec![
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage{
                        content: String::from("Tell me a random fun fact about Human speech").into(),
                        name: None,
                })
        ]).build()
        .unwrap();

        let chat_stream_resp = chat.create_stream(chat_req).await.unwrap();
        pin_mut!(chat_stream_resp);

        for await chunk in chat_stream_resp {
            let choices = chunk.unwrap().choices;
            for choice in choices {
                let delta = choice.delta;
                if let Some(content) = delta.content {
                    // first "content" is an empty string
                    // an empty string indicates the `EOSMessage` end of the sequence message in elevenlabs websocket protocol
                    // so we skip it, and any other empty strings if such things occur.
                    if content.is_empty() {
                        continue;
                    }
                    yield content;
                }
            }
        }
    };

    let body = WebSocketTTSBody::new(BOSMessage::default(), text_stream);
    let endpoint = WebSocketTTS::new(voice_id, model_id, body);
    let stream = elevenlabs_client.hit_ws(endpoint).await?;

    stream_audio(stream.map(|r| r?.audio_as_bytes())).await?;

    Ok(())
}
