use elevenlabs_rs::endpoints::admin::pronunciation::*;
use elevenlabs_rs::endpoints::genai::tts::*;
use elevenlabs_rs::utils::play;
use elevenlabs_rs::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ElevenLabsClient::from_env()?;

    let txt = "I'm using a TTS model via the ElevenLabs' API to say tomato and Tomato.";
    let model = Model::ElevenTurboV2;
    let voice_id = DefaultVoice::Alice;

    let mut tts_body = TextToSpeechBody::new(txt).with_model_id(model.clone());
    let mut tts_endpoint = TextToSpeech::new(voice_id.clone(), tts_body);
    let speech_without_dict = client.hit(tts_endpoint).await?;

    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dictionary.pls");
    let body = CreateDictionaryBody::new(file.to_str().unwrap(), "example");
    let resp = client.hit(CreateDictionary::new(body)).await?;
    let mut dictionary_id = resp.id;
    let mut version_id = resp.version_id;

    let mut pls_bytes = client
        .hit(GetPLSFile::new(&dictionary_id, &version_id))
        .await?;

    let mut current_dictionary_state = unsafe { std::str::from_utf8_unchecked(&pls_bytes) };
    println!("--- Initial dictionary ---\n");
    println!("{}\n", current_dictionary_state);

    let mut locators = DictionaryLocators::default();
    locators.push(DictionaryLocator::new(&dictionary_id, &version_id));

    tts_body = TextToSpeechBody::new(txt)
        .with_model_id(model.clone())
        .with_dictionary_locators(locators.clone());

    tts_endpoint = TextToSpeech::new(voice_id.clone(), tts_body);
    let speech_with_dict = client.hit(tts_endpoint).await?;

    let rules = vec![
        Rule::new_alias("TTS", "text to speech"),
        Rule::new_alias("API", "application programming interface"),
        Rule::new_phoneme("via", "/ˈvaɪə/", "ipa"),
    ];

    let body = AddRulesBody::new(rules);
    let mut resp = client.hit(AddRules::new(dictionary_id, body)).await?;
    dictionary_id = resp.id;
    version_id = resp.version_id;

    pls_bytes = client
        .hit(GetPLSFile::new(&dictionary_id, &version_id))
        .await?;

    current_dictionary_state = unsafe { std::str::from_utf8_unchecked(&pls_bytes) };
    println!("--- Dictionary with added rules ---\n");
    println!("{}\n", current_dictionary_state);

    locators = Default::default();
    locators.push(DictionaryLocator::new(&dictionary_id, &version_id));

    tts_body = TextToSpeechBody::new(txt)
        .with_model_id(model.clone())
        .with_dictionary_locators(locators);

    tts_endpoint = TextToSpeech::new(voice_id.clone(), tts_body);
    let speech_with_added_rules = client.hit(tts_endpoint).await?;

    let remove_rules = vec!["TTS", "tomato", "Tomato"];
    let body = RemoveRulesBody::new(remove_rules);

    resp = client.hit(RemoveRules::new(dictionary_id, body)).await?;

    dictionary_id = resp.id;
    version_id = resp.version_id;

    pls_bytes = client
        .hit(GetPLSFile::new(&dictionary_id, &version_id))
        .await?;
    current_dictionary_state = unsafe { std::str::from_utf8_unchecked(&pls_bytes) };
    println!("--- Dictionary with removed rules ---\n");
    println!("{}\n", current_dictionary_state);

    locators = Default::default();
    locators.push(DictionaryLocator::new(&dictionary_id, &version_id));

    tts_body = TextToSpeechBody::new(txt)
        .with_model_id(model.clone())
        .with_dictionary_locators(locators);

    tts_endpoint = TextToSpeech::new(voice_id, tts_body);
    let speech_with_removed_rules = client.hit(tts_endpoint).await?;

    play(speech_without_dict)?;
    play(speech_with_dict)?;
    play(speech_with_added_rules)?;
    play(speech_with_removed_rules)?;

    Ok(())
}
