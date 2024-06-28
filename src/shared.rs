pub mod identifiers {
    use serde::Serialize;
    use crate::endpoints::Deserialize;




    #[derive(Clone, Debug, Deserialize)]
    pub(crate) struct HistoryItemID(pub(crate)String);

    impl From<String> for HistoryItemID {
        fn from(id: String) -> Self {
            Self(id)
        }
    }

    #[derive(Clone, Debug, Default, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub(crate) struct ModelID(pub(crate) String);

    impl From<String> for ModelID {
        fn from(id: String) -> Self {
            ModelID(id)
        }
    }

    #[derive(Clone, Debug)]
    pub enum Model {
        ElevenMultilingualV2,
        ElevenMultilingualV1,
        ElevenEnglishV1,
        ElevenEnglishV2,
        ElevenTurboV2,
        ElevenMultilingualV2STS,
    }

    impl From<Model> for String {
        fn from(model: Model) -> String {
            match model {
                Model::ElevenMultilingualV2 => "eleven_multilingual_v2".to_string(),
                Model::ElevenMultilingualV1 => "eleven_multilingual_v1".to_string(),
                Model::ElevenEnglishV1 => "eleven_monolingual_v1".to_string(),
                Model::ElevenEnglishV2 => "eleven_english_sts_v2".to_string(),
                Model::ElevenTurboV2 => "eleven_turbo_v2".to_string(),
                Model::ElevenMultilingualV2STS => "eleven_multilingual_sts_v2".to_string(),
            }
        }
    }



    #[derive(Clone, Debug)]
    pub(crate) struct VoiceID(pub(crate) String);

    impl From<String> for VoiceID {
        fn from(id: String) -> Self {
            VoiceID(id)
        }
    }

    #[derive(Clone, Debug, Default)]
    pub enum PreMadeVoiceID {
        Adam,
        Alice,
        Antoni,
        Arnold,
        Bill,
        Brian,
        Callum,
        Charlie,
        Chris,
        Clyde,
        Daniel,
        Dave,
        Dorothy,
        Drew,
        Domi,
        Eli,
        Emily,
        Ethan,
        Fin,
        Freya,
        George,
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
        Liam,
        Lily,
        Matilda,
        Michael,
        Mimi,
        Nicole,
        Patrick,
        Paul,
        #[default]
        Rachel,
        Sam,
        Sarah,
        Serena,
        Thomas,
    }

    impl From<PreMadeVoiceID> for String {
        fn from(id: PreMadeVoiceID) -> String {
            match id {
                PreMadeVoiceID::Adam => "pNInz6obpgDQGcFmaJgB".to_string(),
                PreMadeVoiceID::Alice => "Xb7hH8MSUJpSbSDYk0k2".to_string(),
                PreMadeVoiceID::Antoni => "ErXwobaYiN019PkySvjV".to_string(),
                PreMadeVoiceID::Arnold => "VR6AewLTigWG4xSOukaG".to_string(),
                PreMadeVoiceID::Bill => "pqHfZKP75CvOlQylNhV4".to_string(),
                PreMadeVoiceID::Brian => "nPczCjzI2devNBz1zQrb".to_string(),
                PreMadeVoiceID::Callum => "N2lVS1w4EtoT3dr4eOWO".to_string(),
                PreMadeVoiceID::Charlie => "IKne3meq5aSn9XLyUdCD".to_string(),
                PreMadeVoiceID::Chris => "iP95p4xoKVk53GoZ742B".to_string(),
                PreMadeVoiceID::Clyde => "2EiwWnXFnvU5JabPnv8n".to_string(),
                PreMadeVoiceID::Daniel => "onwK4e9ZLuTAKqWW03F9".to_string(),
                PreMadeVoiceID::Dave => "CYw3kZ02Hs0563khs1Fj".to_string(),
                PreMadeVoiceID::Dorothy => "ThT5KcBeYPX3keUQqHPh".to_string(),
                PreMadeVoiceID::Drew => "29vD33N1CtxCmqQRPOHJ".to_string(),
                PreMadeVoiceID::Domi => "AZnzlk1XvdvUeBnXmlld".to_string(),
                PreMadeVoiceID::Eli => "MF3mGyEYCl7XYWbV9V6O".to_string(),
                PreMadeVoiceID::Emily => "LcfcDJNUP1GQjkzn1xUU".to_string(),
                PreMadeVoiceID::Ethan => "g5CIjZEefAph4nQFvHAz".to_string(),
                PreMadeVoiceID::Fin => "D38z5RcWu1voky8WS1ja".to_string(),
                PreMadeVoiceID::Freya => "jsCqWAovK2LkecY7zXl4".to_string(),
                PreMadeVoiceID::George => "JBFqnCBsd6RMkjVDRZzb".to_string(),
                PreMadeVoiceID::Gigi => "jBpfuIE2acCO8z3wKNLl".to_string(),
                PreMadeVoiceID::Giovanni => "zcAOhNBS3c14rBihAFp1".to_string(),
                PreMadeVoiceID::Glinda => "z9fAnlkpzviPz146aGWa".to_string(),
                PreMadeVoiceID::Grace => "oWAxZDx7w5VEj9dCyTzz".to_string(),
                PreMadeVoiceID::Harry => "SOYHLrjzK2X1ezoPC6cr".to_string(),
                PreMadeVoiceID::James => "ZQe5CZNOzWyzPSCn5a3c".to_string(),
                PreMadeVoiceID::Jessie => "t0jbNlBVZ17f02VDIeMI".to_string(),
                PreMadeVoiceID::Jeremy => "bVMeCyTHy58xNoL34h3p".to_string(),
                PreMadeVoiceID::Joseph => "Zlb1dXrM653N07WRdFW3".to_string(),
                PreMadeVoiceID::Josh => "TxGEqnHWrfWFTfGW9XjX".to_string(),
                PreMadeVoiceID::Liam => "TX3LPaxmHKxFdv7VOQHJ".to_string(),
                PreMadeVoiceID::Lily => "pFZP5JQG7iQjIQuC4Bku".to_string(),
                PreMadeVoiceID::Matilda => "XrExE9yKIg1WjnnlVkGX".to_string(),
                PreMadeVoiceID::Michael => "flq6f7yk4E4fJM5XTYuZ".to_string(),
                PreMadeVoiceID::Mimi => "zrHiDhphv9ZnVXBqCLjz".to_string(),
                PreMadeVoiceID::Nicole => "piTKgcLEGmPE4e6mEKli".to_string(),
                PreMadeVoiceID::Patrick => "ODq5zmih8GrVes37Dizd".to_string(),
                PreMadeVoiceID::Paul => "5Q0t7uMcjvnagumLfvZi".to_string(),
                PreMadeVoiceID::Rachel => "21m00Tcm4TlvDq8ikWAM".to_string(),
                PreMadeVoiceID::Sam => "yoZ06aMxZJJ28mfd3POQ".to_string(),
                PreMadeVoiceID::Sarah => "EXAVITQu4vr4xnSDxMaL".to_string(),
                PreMadeVoiceID::Serena => "pMsXgVXv3BLzUgSXRplE".to_string(),
                PreMadeVoiceID::Thomas => "GBv7mTt0atIp3Br8iCZE".to_string(),
            }
        }
    }
}

pub(crate) mod path_segments {
    pub(crate) const ADD_VOICE_PATH: &str = "/add";
    pub(crate) const DOWNLOAD_PATH: &str = "/download";
    pub(crate) const STREAM_PATH: &str = "/stream";
    pub(crate) const VOICES_PATH: &str = "/v1/voices";
}

pub mod response_bodies {
    use serde::Deserialize;
    #[derive(Clone, Debug, Deserialize)]
    pub struct StatusResponseBody {
        pub status: String,
    }
}

pub mod query_params {
    #[derive(Clone, Debug)]
    pub enum Latency {
        /// Default latency
        None = 0,
        ///  normal latency optimizations (about 50% of possible latency improvement of option 3)
        Normal = 1,
        /// strong latency optimizations (about 75% of possible latency improvement of option 3)
        Strong = 2,
        /// max latency optimizations
        Max = 3,
        /// max latency optimizations, but also with text normalizer turned off for even more latency
        /// savings (the best latency, but can mispronounce e.g. numbers and dates)
        MaxBest = 4,
    }

    /// See Elevenlabs documentation on [supported output formats](https://help.elevenlabs.io/hc/en-us/articles/15754340124305-What-audio-formats-do-you-support).
    #[derive(Clone, Debug)]
    pub enum OutputFormat {
        Mp3_22050Hz32kbps,
        Mp3_44100Hz32kbps,
        Mp3_44100Hz64kbps,
        Mp3_44100Hz96kbps,
        Mp3_44100Hz192kbps,
        Pcm16000Hz,
        Pcm22050Hz,
        Pcm24000Hz,
        Pcm44100Hz,
        MuLaw8000Hz,
    }
    impl OutputFormat {
        pub(crate) fn to_query(&self) -> &str {
            match self {
                OutputFormat::Pcm16000Hz => "pcm_16000",
                OutputFormat::Pcm22050Hz => "pcm_22050",
                OutputFormat::Pcm24000Hz => "pcm_24000",
                OutputFormat::Pcm44100Hz => "pcm_44100",
                OutputFormat::Mp3_22050Hz32kbps => "mp3_22050_32",
                OutputFormat::Mp3_44100Hz32kbps => "mp3_44100_32",
                OutputFormat::Mp3_44100Hz64kbps => "mp3_44100_64",
                OutputFormat::Mp3_44100Hz96kbps => "mp3_44100_96",
                OutputFormat::Mp3_44100Hz192kbps => "mp3_44100_192",
                OutputFormat::MuLaw8000Hz => "ulaw_8000",
            }
        }
    }
}
