use crate::{
    api::{voice::VoiceSettings, Client, ClientBuilder},
    error::Error,
    prelude::*,
    utils::save,
};
use chrono::{Local, LocalResult, TimeZone};
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;

const BASE_PATH: &str = "/history";
const AUDIO_PATH: &str = "/audio";
const DOWNLOAD_PATH: &str = "/download";
const PAGE_SIZE_QUERY: &str = "page_size";
const HISTORY_ITEM_IDS: &str = "history_item_ids";
const START_AFTER_HISTORY_ITEM_ID_QUERY: &str = "start_after_history_item_id";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::tts::Speech;
    use std::any::{Any, TypeId};

    #[tokio::test]
    #[ignore]
    async fn get_history_is_returning_a_history_checked_by_type_id() {
        let want = TypeId::of::<History>();
        let got = get_history(Some(1), None).await.unwrap().type_id();
        assert_eq!(want, got)
    }

    #[tokio::test]
    #[ignore]
    async fn history_items_delete_is_deleting_that_history_item() {
        let _speech = Speech::new("test", "Adam", "eleven_monolingual_v1", 4)
            .await
            .unwrap();

        let mut h = get_history(Some(1), None).await.unwrap();

        let mut history_item = get_history_item(&h.history[0].history_item_id)
            .await
            .unwrap();

        assert_eq!(&history_item.text, "test");

        let _ = delete_history_item(&history_item.history_item_id)
            .await
            .unwrap();

        h = get_history(Some(1), None).await.unwrap();

        history_item = get_history_item(&h.history[0].history_item_id)
            .await
            .unwrap();

        assert_ne!(&history_item.text, "test");
    }

    #[test]
    fn character_count_change_is_subtracting_character_count_to_from_character_count_from() {
        let want = 169;
        let history_item = HistoryItem {
            character_count_change_from: 71155,
            character_count_change_to: 71324,
            ..Default::default()
        };
        let got = history_item.character_count_change();
        assert_eq!(want, got)
    }
    #[test]
    fn datetime_is_converting_date_unix_to_a_local_datetime_with_format_of_dd_mm_yyyy_hh_mm() {
        let want = "28-07-2023 16:05";
        let history_item = HistoryItem {
            date_unix: 1690556729,
            ..Default::default()
        };
        let got = history_item.datetime().unwrap();
        assert_eq!(want, got)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HistoryItem {
    pub history_item_id: String,
    pub request_id: String,
    pub voice_id: String,
    pub voice_name: String,
    pub text: String,
    pub date_unix: i64,
    pub character_count_change_from: i64,
    pub character_count_change_to: i64,
    pub content_type: String,
    pub state: String,
    pub settings: VoiceSettings,
    pub feedback: Option<Feedback>,
}

impl HistoryItem {
    /// Gets the audio of the history item
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::api::history::*;
    /// use elevenlabs_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///    let h = get_history(Some(1), None).await?;
    ///    let history_item = h.history.first().unwrap();
    ///    let audio = history_item.get_audio().await?;
    ///    
    ///    // do something with the audio
    ///
    ///   Ok(())
    /// }
    /// ```
    pub async fn get_audio(&self) -> Result<Bytes> {
        let data = get_history_audio_item(&self.history_item_id).await?;
        Ok(data)
    }

    /// Deletes the history item
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::api::history::*;
    /// use elevenlabs_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let h = get_history(Some(1), None).await?;
    ///     let history_item = h.history.first().unwrap();
    ///     history_item.delete().await?;
    ///
    /// Ok(())
    /// }
    /// ```
    pub async fn delete(&self) -> Result<()> {
        let _ = delete_history_item(&self.history_item_id).await?;
        Ok(())
    }

    /// Saves the audio of the history item
    /// if no filename is provided, it will use \[voice_name\]_\[date_unix\].mp3
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::api::history::*;
    /// use elevenlabs_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///   let h = get_history(Some(1), None).await?;
    ///   let history_item = h.history.first().unwrap();
    ///   history_item.save(None).await?;
    ///
    ///  Ok(())
    /// }
    /// ```
    pub async fn save(&self, filename: Option<String>) -> Result<()> {
        let filename = match filename {
            Some(f) => f,
            None => {
                let mut filename = self.voice_name.clone();
                filename.push_str("_");
                filename.push_str(&self.date_unix.to_string());
                filename.push_str(".mp3");
                filename
            }
        };
        let data = self.get_audio().await?;
        save(&filename, data)?;
        Ok(())
    }

    /// Returns the character count change
    /// # Example
    /// ```no_run
    /// use elevenlabs_rs::api::history::*;
    /// use elevenlabs_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///    let h = get_history(Some(1), None).await?;
    ///    let item = h.history.first().unwrap();
    ///    println!("{:?}", item.character_count_change());
    ///
    ///    Ok(())
    /// }
    /// ```
    pub fn character_count_change(&self) -> i64 {
        self.character_count_change_to - self.character_count_change_from
    }

    pub fn datetime(&self) -> Result<String> {
        let dt = Local.timestamp_opt(self.date_unix, 0);
        match dt {
            LocalResult::Single(dt) => Ok(dt.format("%d-%m-%Y %H:%M").to_string()),
            LocalResult::Ambiguous(_, _) => Err(Box::new(Error::InvalidTimestamp(
                self.date_unix.to_string(),
            ))),
            LocalResult::None => Err(Box::new(Error::InvalidTimestamp(
                self.date_unix.to_string(),
            ))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct History {
    pub history: Vec<HistoryItem>,
    pub last_history_item_id: String,
    pub has_more: bool,
}

impl History {
    pub fn items(&self) -> Vec<HistoryItem> {
        self.history.clone()
    }

    pub async fn iter(&mut self) -> Result<impl Iterator<Item = &HistoryItem>> {
        while self.has_more {
            let mut more_history =
                get_history(Some(1000), Some(&self.last_history_item_id)).await?;
            self.history.append(&mut more_history.history);
            self.last_history_item_id = more_history.last_history_item_id;
            self.has_more = more_history.has_more;
        }
        Ok(self.history.iter())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Feedback {
    pub thumbs_up: bool,
    pub feedback: String,
    pub emotions: bool,
    pub inaccurate_clone: bool,
    pub glitches: bool,
    pub audio_quality: bool,
    pub review_status: String,
}

pub fn build_history_client() -> Result<Client> {
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(BASE_PATH)?
        .method(GET)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    Ok(c)
}

/// page_size determines how many history items to return at maximum. Can not exceed 1000, defaults to 100.
///
/// start_after_history_item_id determines the point from which to start fetching the items,
/// use this parameter to paginate across a large collection of history items.
/// In case this parameter is not provided history items will be fetched starting from the most recently created one ordered descending by their creation date.
///
/// # Example
///
/// ```no_run
///
/// use elevenlabs_rs::api::history::*;
/// use elevenlabs_rs::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    // Returns a History with the first 10 history items that have been recently created
///    let h = get_history(Some(10), None).await?;
///     
///    // Returns a History with a max of 100 history items that have been recently created
///   let h = get_history(None, None).await?;
///
///   Ok(())
/// }
/// ```
pub async fn get_history(
    page_size: Option<usize>,
    start_after_history_item_id: Option<&str>,
) -> Result<History> {
    let path_with_query = match (page_size, start_after_history_item_id) {
        (Some(ps), Some(sahi)) => format!(
            "{}?{}={}&{}={}",
            BASE_PATH,
            PAGE_SIZE_QUERY,
            &ps.to_string(),
            START_AFTER_HISTORY_ITEM_ID_QUERY,
            sahi
        ),
        (Some(ps), None) => format!("{}?{}={}", BASE_PATH, PAGE_SIZE_QUERY, &ps.to_string()),
        (None, Some(sahi)) => {
            format!(
                "{}?{}={}",
                BASE_PATH, START_AFTER_HISTORY_ITEM_ID_QUERY, sahi
            )
        }
        (None, None) => BASE_PATH.to_string(),
    };

    let cb = ClientBuilder::new()?;
    let c = cb
        .path(path_with_query)?
        .method(GET)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let history: History = serde_json::from_slice(resp.as_ref())?;
    Ok(history)
}

pub async fn get_history_item(history_item_id: impl Into<String>) -> Result<HistoryItem> {
    let cb = ClientBuilder::new()?;
    let id = history_item_id.into();
    let c = cb
        .path(format!("{}/{}", BASE_PATH, id))?
        .method(GET)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let resp = c.send_request(Empty::<Bytes>::new()).await?;
    let history_item: HistoryItem = serde_json::from_slice(resp.as_ref())?;
    Ok(history_item)
}

pub async fn get_history_audio_item(history_item_id: impl Into<String>) -> Result<Bytes> {
    let cb = ClientBuilder::new()?;
    let id = history_item_id.into();
    let c = cb
        .path(format!("{}/{}{}", BASE_PATH, id, AUDIO_PATH))?
        .method(GET)?
        .header(ACCEPT, AUDIO_MPEG)?
        .build()?;
    let data = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(data)
}

pub async fn delete_history_item(id: impl Into<String>) -> Result<()> {
    let cb = ClientBuilder::new()?;
    let id = id.into();
    let c = cb
        .path(format!("{}/{}", BASE_PATH, id))?
        .method(DELETE)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let _data = c.send_request(Empty::<Bytes>::new()).await?;
    Ok(())
}

/// Download one or more history items. If one history item ID is provided,
/// ElevenLabs returns a single audio file. If more than one history item IDs are provided,
/// ElevenLabs provides the history items packed into a .zip file.
pub async fn download_history_items(ids: Vec<&str>) -> Result<Bytes> {
    let body = json!({
            HISTORY_ITEM_IDS: ids,})
    .to_string();
    let cb = ClientBuilder::new()?;
    let c = cb
        .path(format!("{}{}", BASE_PATH, DOWNLOAD_PATH))?
        .method(POST)?
        .header(ACCEPT, APPLICATION_JSON)?
        .build()?;
    let data = c.send_request(Full::<Bytes>::new(body.into())).await?;
    Ok(data)
}
