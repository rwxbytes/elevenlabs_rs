
pub(crate) trait Identifier {
    const PLACEHOLDER: &'static str;
    fn get_value(&self) -> &str;

    fn as_path_param(&self) -> (&'static str, &str) {
        (Self::PLACEHOLDER, self.get_value())
    }
}

#[derive(Clone, Debug)]
pub struct DictionaryID {
    _inner: String,
}

impl Identifier for DictionaryID {
    const PLACEHOLDER: &'static str = ":pronunciation_dictionary_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for DictionaryID {
    fn from(id: String) -> Self {
        DictionaryID { _inner: id }
    }
}

impl From<&String> for DictionaryID {
    fn from(id: &String) -> Self {
        DictionaryID { _inner: id.clone() }
    }
}

impl From<&str> for DictionaryID {
    fn from(id: &str) -> Self {
        DictionaryID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DubbingID {
 pub(crate)  _inner: String,
}

impl Identifier for DubbingID {
    const PLACEHOLDER: &'static str = ":dubbing_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for DubbingID {
    fn from(id: String) -> Self {
        DubbingID { _inner: id }
    }
}

impl From<&String> for DubbingID {
    fn from(id: &String) -> Self {
        DubbingID { _inner: id.clone() }
    }
}

impl From<&str> for DubbingID {
    fn from(id: &str) -> Self {
        DubbingID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistoryItemID {
    pub _inner: String,
}

impl Identifier for HistoryItemID {
    const PLACEHOLDER: &'static str = ":history_item_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for HistoryItemID {
    fn from(id: String) -> Self {
        Self { _inner: id }
    }
}

impl From<&String> for HistoryItemID {
    fn from(id: &String) -> Self {
        Self { _inner: id.clone() }
    }
}

impl From<&str> for HistoryItemID {
    fn from(id: &str) -> Self {
        Self {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModelID {
    pub _inner: String,
}

impl Identifier for ModelID {
    const PLACEHOLDER: &'static str = ":model_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for ModelID {
    fn from(id: String) -> Self {
        ModelID { _inner: id }
    }
}

impl From<&String> for ModelID {
    fn from(id: &String) -> Self {
        ModelID { _inner: id.clone() }
    }
}

impl From<&str> for ModelID {
    fn from(id: &str) -> Self {
        ModelID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LanguageCodeID {
    pub(crate) _inner: String,
}

impl Identifier for LanguageCodeID {
    const PLACEHOLDER: &'static str = ":language_code";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for LanguageCodeID {
    fn from(id: String) -> Self {
        LanguageCodeID { _inner: id }
    }
}

impl From<&String> for LanguageCodeID {
    fn from(id: &String) -> Self {
        LanguageCodeID { _inner: id.clone() }
    }
}

impl From<&str> for LanguageCodeID {
    fn from(id: &str) -> Self {
        LanguageCodeID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublicUserID {
    _inner: String,
}

impl Identifier for PublicUserID {
    const PLACEHOLDER: &'static str = ":public_user_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for PublicUserID {
    fn from(id: String) -> Self {
        PublicUserID { _inner: id }
    }
}

impl From<&String> for PublicUserID {
    fn from(id: &String) -> Self {
        PublicUserID { _inner: id.clone() }
    }
}

impl From<&str> for PublicUserID {
    fn from(id: &str) -> Self {
        PublicUserID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SampleID {
    pub _inner: String,
}

impl Identifier for SampleID {
    const PLACEHOLDER: &'static str = ":sample_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for SampleID {
    fn from(id: String) -> Self {
        Self { _inner: id }
    }
}

impl From<&String> for SampleID {
    fn from(id: &String) -> Self {
        SampleID { _inner: id.clone() }
    }
}

impl From<&str> for SampleID {
    fn from(id: &str) -> Self {
        Self {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VersionID {
    _inner: String,
}

impl Identifier for VersionID {
    const PLACEHOLDER: &'static str = ":version_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for VersionID {
    fn from(id: String) -> Self {
        VersionID { _inner: id }
    }
}

impl From<&String> for VersionID {
    fn from(id: &String) -> Self {
        VersionID { _inner: id.clone() }
    }
}

impl From<&str> for VersionID {
    fn from(id: &str) -> Self {
        VersionID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VoiceID {
    pub _inner: String,
}

impl Identifier for VoiceID {
    const PLACEHOLDER: &'static str = ":voice_id";
    fn get_value(&self) -> &str {
        &self._inner
    }
}

impl From<String> for VoiceID {
    fn from(id: String) -> Self {
        VoiceID { _inner: id }
    }
}

impl From<&String> for VoiceID {
    fn from(id: &String) -> Self {
        VoiceID { _inner: id.clone() }
    }
}

impl From<&str> for VoiceID {
    fn from(id: &str) -> Self {
        VoiceID {
            _inner: id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Model {
    ElevenMultilingualV2,
    #[deprecated]
    ElevenMultilingualV1,
    #[deprecated]
    ElevenEnglishV1,
    ElevenEnglishV2,
    ElevenTurboV2,
    ElevenTurboV2_5,
    ElevenMultilingualV2STS,
    ElevenFlashV2,
    ElevenFlashV2_5,
}

impl From<Model> for String {
    fn from(model: Model) -> String {
        match model {
            Model::ElevenMultilingualV2 => "eleven_multilingual_v2".to_string(),
            Model::ElevenMultilingualV1 => "eleven_multilingual_v1".to_string(),
            Model::ElevenEnglishV1 => "eleven_monolingual_v1".to_string(),
            Model::ElevenEnglishV2 => "eleven_english_sts_v2".to_string(),
            Model::ElevenTurboV2 => "eleven_turbo_v2".to_string(),
            Model::ElevenTurboV2_5 => "eleven_turbo_v2_5".to_string(),
            Model::ElevenMultilingualV2STS => "eleven_multilingual_sts_v2".to_string(),
            Model::ElevenFlashV2 => "eleven_flash_v2".to_string(),
            Model::ElevenFlashV2_5 => "eleven_flash_v2_5".to_string(),
        }
    }
}

impl From<Model> for ModelID {
    fn from(model: Model) -> ModelID {
        ModelID {
            _inner: String::from(model),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum DefaultVoice {
    Aria,
    Roger,
    Sarah,
    Laura,
    Charlie,
    George,
    Callum,
    River,
    Liam,
    Charlotte,
    Alice,
    Matilda,
    Will,
    Jessica,
    #[default]
    Eric,
    Chris,
    Brian,
    Daniel,
    Lily,
    Bill,
}

impl From<DefaultVoice> for String {
    fn from(id: DefaultVoice) -> String {
        match id {
            DefaultVoice::Aria => "9BWtsMINqrJLrRacOk9x".to_string(),
            DefaultVoice::Roger => "CwhRBWXzGAHq8TQ4Fs17".to_string(),
            DefaultVoice::Sarah => "EXAVITQu4vr4xnSDxMaL".to_string(),
            DefaultVoice::Laura => "FGY2WhTYpPnrIDTdsKH5".to_string(),
            DefaultVoice::Charlie => "IKne3meq5aSn9XLyUdCD".to_string(),
            DefaultVoice::George => "JBFqnCBsd6RMkjVDRZzb".to_string(),
            DefaultVoice::Callum => "N2lVS1w4EtoT3dr4eOWO".to_string(),
            DefaultVoice::River => "SAz9YHcvj6GT2YYXdXww".to_string(),
            DefaultVoice::Liam => "TX3LPaxmHKxFdv7VOQHJ".to_string(),
            DefaultVoice::Charlotte => "XB0fDUnXU5powFXDhCwa".to_string(),
            DefaultVoice::Alice => "Xb7hH8MSUJpSbSDYk0k2".to_string(),
            DefaultVoice::Matilda => "XrExE9yKIg1WjnnlVkGX".to_string(),
            DefaultVoice::Will => "bIHbv24MWmeRgasZH58o".to_string(),
            DefaultVoice::Jessica => "cgSgspJ2msm6clMCkdW9".to_string(),
            DefaultVoice::Eric => "cjVigY5qzO86Huf0OWal".to_string(),
            DefaultVoice::Chris => "iP95p4xoKVk53GoZ742B".to_string(),
            DefaultVoice::Brian => "nPczCjzI2devNBz1zQrb".to_string(),
            DefaultVoice::Daniel => "onwK4e9ZLuTAKqWW03F9".to_string(),
            DefaultVoice::Lily => "pFZP5JQG7iQjIQuC4Bku".to_string(),
            DefaultVoice::Bill => "pqHfZKP75CvOlQylNhV4".to_string(),
        }
    }
}

impl From<DefaultVoice> for VoiceID {
    fn from(id: DefaultVoice) -> VoiceID {
        VoiceID {
            _inner: String::from(id),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LegacyVoice {
    Adam,
    Antoni,
    Arnold,
    Clyde,
    Dave,
    Dorothy,
    Drew,
    Domi,
    Eli,
    Emily,
    Ethan,
    Fin,
    Freya,
    Gigi,
    Giovanni,
    Glinda,
    Grace,
    Harry,
    James,
    Jessie,
    Jeremy,
    Joseph,
    Josh,
    Michael,
    Mimi,
    Nicole,
    Patrick,
    Paul,
    Rachel,
    Sam,
    Serena,
    Thomas,
}

impl From<LegacyVoice> for String {
    fn from(id: LegacyVoice) -> String {
        match id {
            LegacyVoice::Adam => "pNInz6obpgDQGcFmaJgB".to_string(),
            LegacyVoice::Antoni => "ErXwobaYiN019PkySvjV".to_string(),
            LegacyVoice::Arnold => "VR6AewLTigWG4xSOukaG".to_string(),
            LegacyVoice::Clyde => "2EiwWnXFnvU5JabPnv8n".to_string(),
            LegacyVoice::Dave => "CYw3kZ02Hs0563khs1Fj".to_string(),
            LegacyVoice::Dorothy => "ThT5KcBeYPX3keUQqHPh".to_string(),
            LegacyVoice::Drew => "29vD33N1CtxCmqQRPOHJ".to_string(),
            LegacyVoice::Domi => "AZnzlk1XvdvUeBnXmlld".to_string(),
            LegacyVoice::Eli => "MF3mGyEYCl7XYWbV9V6O".to_string(),
            LegacyVoice::Emily => "LcfcDJNUP1GQjkzn1xUU".to_string(),
            LegacyVoice::Ethan => "g5CIjZEefAph4nQFvHAz".to_string(),
            LegacyVoice::Fin => "D38z5RcWu1voky8WS1ja".to_string(),
            LegacyVoice::Freya => "jsCqWAovK2LkecY7zXl4".to_string(),
            LegacyVoice::Gigi => "jBpfuIE2acCO8z3wKNLl".to_string(),
            LegacyVoice::Giovanni => "zcAOhNBS3c14rBihAFp1".to_string(),
            LegacyVoice::Glinda => "z9fAnlkpzviPz146aGWa".to_string(),
            LegacyVoice::Grace => "oWAxZDx7w5VEj9dCyTzz".to_string(),
            LegacyVoice::Harry => "SOYHLrjzK2X1ezoPC6cr".to_string(),
            LegacyVoice::James => "ZQe5CZNOzWyzPSCn5a3c".to_string(),
            LegacyVoice::Jessie => "t0jbNlBVZ17f02VDIeMI".to_string(),
            LegacyVoice::Jeremy => "bVMeCyTHy58xNoL34h3p".to_string(),
            LegacyVoice::Joseph => "Zlb1dXrM653N07WRdFW3".to_string(),
            LegacyVoice::Josh => "TxGEqnHWrfWFTfGW9XjX".to_string(),
            LegacyVoice::Michael => "flq6f7yk4E4fJM5XTYuZ".to_string(),
            LegacyVoice::Mimi => "zrHiDhphv9ZnVXBqCLjz".to_string(),
            LegacyVoice::Nicole => "piTKgcLEGmPE4e6mEKli".to_string(),
            LegacyVoice::Patrick => "ODq5zmih8GrVes37Dizd".to_string(),
            LegacyVoice::Paul => "5Q0t7uMcjvnagumLfvZi".to_string(),
            LegacyVoice::Rachel => "21m00Tcm4TlvDq8ikWAM".to_string(),
            LegacyVoice::Sam => "yoZ06aMxZJJ28mfd3POQ".to_string(),
            LegacyVoice::Serena => "pMsXgVXv3BLzUgSXRplE".to_string(),
            LegacyVoice::Thomas => "GBv7mTt0atIp3Br8iCZE".to_string(),
        }
    }
}

impl From<LegacyVoice> for VoiceID {
    fn from(id: LegacyVoice) -> VoiceID {
        VoiceID {
            _inner: String::from(id),
        }
    }
}
