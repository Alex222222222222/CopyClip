use crate::{
    clip::get_system_timestamp,
    config::ConfigMutex,
    error::Error,
    event::{CopyClipEvent, EventSender},
};
use std::sync::Arc;

use super::{
    cache::CACHED_CLIP, clip_id::ClipID, clip_struct::Clip, clip_type::ClipType, pinned::PinnedClip,
};
use log::debug;
use once_cell::sync::Lazy;
use sqlx::{Row, SqlitePool};
use tauri::{async_runtime::Mutex, AppHandle, ClipboardManager, Manager};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Clips {
    /// the id of the current clip
    pub current_clip: ClipID,
    /// the current page
    pub current_page: usize,
    /// the ids of all the clips, well sorted               
    pub whole_list_of_ids: Vec<i64>,
    /// the ids of the current displayed clips, the same order with the order displayed in the tray     
    pub tray_ids_map: Vec<i64>,
    /// the pinned clips
    pub pinned_clips: Vec<PinnedClip>,
}

impl Default for Clips {
    fn default() -> Self {
        Self {
            current_clip: ClipID::None,
            current_page: 0,
            whole_list_of_ids: Default::default(),
            tray_ids_map: Default::default(),
            pinned_clips: Default::default(),
        }
    }
}

/// The clip data to be shared between threads
///
/// The database connection could be get like this
/// ```rust
/// let db_connection_mutex = app.state::<ClipData>();
/// let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
/// let db_connection = db_connection_mutex.clone().unwrap();
/// // drop the mutex, to release the lock
/// drop(db_connection_mutex);
/// let res = sqlx::query("UPDATE clips SET favourite = ? WHERE id = ?")
///     .bind(target)
///     .bind(id)
///     .fetch_optional(db_connection.as_ref())
///     .await;
/// ```
#[derive(Debug, Default)]
pub struct ClipData {
    /// the clips data
    pub clips: Mutex<Clips>,
    /// monitor the clipboard
    clipboard_monitor_last_clip: Mutex<Arc<String>>,
    /// the database connection for the normal process
    pub database_connection: Mutex<Option<Arc<SqlitePool>>>,
}

impl ClipData {
    /// Get the id of the latest clip,
    ///
    /// If there is no clip, return None.
    ///
    /// Will try lock the `clips`
    pub async fn get_latest_clip_id(&self) -> Option<i64> {
        let clips = self.clips.lock().await;
        if clips.whole_list_of_ids.is_empty() {
            return None;
        }
        let id = *clips.whole_list_of_ids.last()?;
        if id < 0 {
            return None;
        }
        Some(id)
    }

    /// change the clip favourite state to the target state
    ///     - change the clip in the cache
    ///     - change the clip in the database
    ///
    /// Will try lock `database_connection`.
    pub async fn change_favourite_clip(&self, id: i64, target: bool) -> Result<(), Error> {
        // change the clip in the cache
        // no need to wait for the result
        CACHED_CLIP.update_favourite_state(id, target).await;

        // change the clip in the database
        // wait for any error
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query("UPDATE clips SET favourite = ? WHERE id = ?")
            .bind(target)
            .bind(id)
            .fetch_optional(db_connection.as_ref())
            .await;

        if let Err(err) = res {
            return Err(Error::UpdateClipsInDatabaseErr(
                format!(
                    "toggle clip favourite state of id: {} to favourite state:{}",
                    id, target
                ),
                err.to_string(),
            ));
        }

        Ok(())
    }

    /// Delete a clip from the database and the cache,
    /// this method will not delete any pinned clip.
    ///
    /// None represent the latest clip.
    ///
    /// Will try lock `database_connection` and `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn delete_normal_clip(&self, mut id: Option<i64>) -> Result<(), Error> {
        // if id is none, change it to the latest clip
        if id.is_none() {
            id = self.get_latest_clip_id().await;
        }
        if id.is_none() {
            return Ok(());
        }
        let id = id.unwrap();

        // delete in cache
        // no need to wait for the result
        CACHED_CLIP.remove(id).await;

        // delete in database
        // wait for any error
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query("DELETE FROM clips WHERE id = ?")
            .bind(id)
            .fetch_optional(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::DeleteClipFromDatabaseErr(id, err.to_string()));
        }

        // delete from the whole list of ids
        // if the clip is the current clip, then change the current clip to most recent clip
        let pos = self.get_id_pos_in_whole_list_of_ids(Some(id)).await;
        if pos.is_none() {
            // no need to delete
            return Ok(());
        }
        let mut clips = self.clips.lock().await;
        let pos = pos.unwrap();
        if pos as usize >= clips.whole_list_of_ids.len() {
            return Ok(());
        }
        if clips.current_clip.is_clip() {
            let id_c = clips.current_clip.get_id();
            if let Some(id_c) = id_c {
                if id == id_c {
                    clips.current_clip = ClipID::None;
                }
            }
        }
        clips.whole_list_of_ids.remove(pos as usize);

        Ok(())
    }

    /// Change the current page to the first page.
    ///
    /// Will try lock `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn first_page(&self) {
        let mut clips = self.clips.lock().await;
        clips.current_page = 0;
    }

    /// Get the clip with id.
    ///
    /// If the given id is none, it will return the latest clip.
    /// If no clip with id exist, it will return None.
    /// The clip id should not be a pinned clip id.
    ///
    /// Will try lock `database_connection` and `clips`.
    #[warn(unused_must_use)]
    pub async fn get_normal_clip(&self, mut id: Option<i64>) -> Result<Option<Clip>, Error> {
        // if id is none, change it to the latest clip
        if id.is_none() {
            id = self.get_latest_clip_id().await;
        }
        if id.is_none() {
            return Ok(None);
        }
        let id = id.unwrap();

        // if the clip is in the cache, return it
        let clip = CACHED_CLIP.get(id).await;
        if let Some(clip) = clip {
            return Ok(Some(clip));
        }

        // if the clip is not in the cache, get it from the database
        // wait for the result
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query("SELECT * FROM clips WHERE id = ?")
            .bind(id)
            .fetch_optional(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }
        let res = res.unwrap();

        if res.is_none() {
            return Ok(None);
        }
        let res = res.unwrap();

        let text = res.try_get::<String, _>("text");
        if let Err(err) = text {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }

        let timestamp = res.try_get::<i64, _>("timestamp");
        if let Err(err) = timestamp {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }

        let id_new = res.try_get::<i64, _>("id");
        if let Err(err) = id_new {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }
        let id = id_new.unwrap();

        let favourite = res.try_get::<i64, _>("favourite");
        if let Err(err) = favourite {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }
        let favourite = favourite.unwrap() == 1;

        let clip = Clip {
            text: Arc::new(text.unwrap()),
            timestamp: timestamp.unwrap(),
            id,
            favourite,
        };

        // add the clip to the cache
        CACHED_CLIP.insert(id, clip.clone()).await;

        Ok(clip.into())
    }

    /// Get a pinned clip with id
    ///
    /// If no clip with id exist, it will try return None.
    ///
    /// Will try lock `clips`.
    #[warn(unused_must_use)]
    pub async fn get_pinned_clip(&self, id: i64) -> Result<Option<PinnedClip>, Error> {
        let clips = self.clips.lock().await;
        for clip in clips.pinned_clips.iter() {
            if clip.id == id {
                return Ok(clip.clone().into());
            }
        }

        Ok(None)
    }

    /// Get a clip by id
    ///
    /// None if the clip is not found.
    ///
    /// Will try lock `database_connection` and `clips`.
    #[warn(unused_must_use)]
    pub async fn get_clip(&self, id: ClipID) -> Result<Option<ClipType>, Error> {
        match id {
            ClipID::Clip(id) => {
                let clip = self.get_normal_clip(Some(id)).await?;
                if clip.is_none() {
                    return Ok(None);
                }
                Ok(Some(ClipType::Clip(clip.unwrap())))
            }
            ClipID::PinnedClip(id) => {
                let clip = self.get_pinned_clip(id).await?;
                if clip.is_none() {
                    return Ok(None);
                }
                Ok(Some(ClipType::PinnedClip(clip.unwrap())))
            }
            ClipID::None => {
                let clip = self.get_normal_clip(None).await?;
                if clip.is_none() {
                    return Ok(None);
                }
                Ok(Some(ClipType::Clip(clip.unwrap())))
            }
        }
    }

    /// Get the position of the clip in the whole list of ids.
    /// Only for normal clips.
    ///
    /// Return the most recent clip if the id is none or the clip is not in the list.
    /// Return None if the list is empty.
    ///
    /// Will try lock `clips`.
    pub async fn get_clip_pos(&self, id: Option<i64>) -> Option<usize> {
        // get the position of the clip in the whole_list_of_ids
        // if the id is not in the list, return the highest pos
        let clip_pos = self.get_id_pos_in_whole_list_of_ids(id).await;
        if clip_pos.is_none() {
            let clips = self.clips.lock().await;
            if clips.whole_list_of_ids.is_empty() {
                return None;
            }
            return Some(clips.whole_list_of_ids.len() - 1);
        }

        Some(clip_pos.unwrap())
    }

    /// Get the clip data if current clip is not pinned clip,
    ///
    /// Will try lock `database_connection` and `clips`.
    pub async fn get_current_clip(&self) -> Result<Option<ClipType>, Error> {
        let clips = self.clips.lock().await;
        let current = clips.current_clip;
        drop(clips);
        self.get_clip(current).await
    }

    /// Get the pos of the current clip if the current clip is not pinned clip.
    ///
    /// None if current clip is pinned clip.
    /// None if the current clip is a normal clip and is not in the list
    /// or the list is empty.
    ///
    /// Will try lock `clips`.
    pub async fn get_current_clip_pos(&self) -> Option<usize> {
        let clips = self.clips.lock().await;
        let current = clips.current_clip;
        drop(clips);

        if current.is_pinned_clip() {
            return None;
        }

        self.get_clip_pos(current.get_id()).await
    }

    /// Get the position of the id in the whole list of ids
    ///
    /// If the id is not in the list, return None.
    /// If the id is none, return None.
    ///
    /// Will try lock `clips`.
    pub async fn get_id_pos_in_whole_list_of_ids(&self, id: Option<i64>) -> Option<usize> {
        let id = id?;

        let clips = self.clips.lock().await;
        let pos = clips.whole_list_of_ids.binary_search(&id);
        if pos.is_err() {
            return None;
        }

        Some(pos.unwrap())
    }

    /// Get the maximum page number depend on
    /// the number of clips and the number of clips per page
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips`.
    pub async fn get_max_page(&self, app: &AppHandle) -> usize {
        // get the max page number
        // if there is no limit, return -1

        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clip_per_page = config.clip_per_page;
        drop(config);
        let clips = self.clips.lock().await;
        if clips.whole_list_of_ids.is_empty() {
            return 0;
        }
        let max_page = clips.whole_list_of_ids.len() / clip_per_page as usize;
        if max_page * clip_per_page as usize == clips.whole_list_of_ids.len() {
            return max_page - 1;
        }
        max_page
    }

    /// Create a new clip in the database and return the id of the new clip
    ///
    /// Will try lock the `clips` and `database_connection` and `app.state::<ConfigMutex>()`.
    pub async fn new_clip(&self, text: Arc<String>, app: &AppHandle) -> Result<i64, Error> {
        let id: i64 = if let Some(id) = self.get_latest_clip_id().await {
            id + 1
        } else {
            1
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let db_connection_mutex = app.state::<ClipData>();
        let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res =
            sqlx::query("INSERT INTO clips (id, text, timestamp, favourite) VALUES (?, ?, ?, ?)")
                .bind(id)
                .bind(&(*text))
                .bind(timestamp)
                .bind(0)
                .fetch_optional(db_connection.as_ref())
                .await;
        if let Err(err) = res {
            return Err(Error::InsertClipIntoDatabaseErr(
                (*text).clone(),
                err.to_string(),
            ));
        }

        let text1 = (*text).clone();
        let clip = Clip {
            text,
            timestamp,
            id,
            favourite: false,
        };

        // add the clip to the cache
        CACHED_CLIP.insert(id, clip).await;

        let mut clips = self.clips.lock().await;
        clips.whole_list_of_ids.push(id);

        // change the current clip to the last one
        clips.current_clip = ClipID::Clip(id);

        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        if !config.auto_delete_duplicate_clip {
            debug!("Auto delete duplicate clip is disabled");
            return Ok(id);
        }
        drop(config);

        debug!("Start auto delete duplicate clip");
        let res = sqlx::query("SELECT id FROM clips WHERE text = ?")
            .bind(&text1)
            .fetch_all(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }
        let res = res.unwrap();
        if res.len() <= 1 {
            return Ok(id);
        }
        let mut ids_to_delete = Vec::new();
        let c = clips.current_clip;
        drop(clips);
        for row in res {
            let id: i64 = row.try_get("id").unwrap();
            if ClipID::Clip(id) == c {
                continue;
            }
            ids_to_delete.push(id);
        }
        for id in ids_to_delete {
            debug!("Delete duplicate clip id: {}", id);
            self.delete_normal_clip(Some(id)).await?;
        }

        Ok(id)
    }

    /// Jump to the next page in the tray
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn next_page(&self, app: &AppHandle) {
        // switch to the next page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await;
        self.switch_page(1, max_page).await;
    }

    /// Jump to the previous page in the tray
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn prev_page(&self, app: &AppHandle) {
        // switch to the previous page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await;
        self.switch_page(-1, max_page).await;
    }

    /// Change the clipboard to the selected clip,
    /// and also change the current clip to the selected clip
    ///
    /// Will try `lock database_connection` and `clips`.
    #[warn(unused_must_use)]
    pub async fn select_clip(&self, app: &AppHandle, id: ClipID) -> Result<(), Error> {
        // try get the clip
        let c = self.get_clip(id).await?;
        if c.is_none() {
            debug!("The requested ClipID: {} is not found", id);
            return Ok(());
        }
        let c = c.unwrap();

        let mut clips = self.clips.lock().await;
        clips.current_clip = id;
        drop(clips);

        c.copy_clip_to_clipboard(app)?;

        Ok(())
    }

    /// Switch the tray to current_page + page,
    /// if the page is < 0, then switch to the first page,
    /// if the page is > max_page, then switch to the last page.
    ///
    /// Will try lock `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn switch_page(&self, page: i64, max_page: usize) {
        // switch to the page by the page number
        // if max_page is -1, it means there is no limit

        let mut clips = self.clips.lock().await;
        let target_page = clips.current_page as i64 + page;
        if target_page < 0 {
            clips.current_page = 0;
            return;
        }
        if target_page > max_page as i64 {
            clips.current_page = max_page;
            return;
        }

        clips.current_page = target_page as usize;
    }

    /// Handle the clip board change event
    ///
    /// If the clipboard text is different from the most recent clip text,
    /// and is different from the current clip text,
    /// and is not empty,
    /// then create a new clip.
    /// Insert the new clip to the database.
    ///
    /// Will try lock `database_connection` and `clips` and `clipboard_monitor_last_clip`
    /// and `app.state::<ConfigMutex>()`.
    ///
    /// Will trigger a tray update event.
    #[warn(unused_must_use)]
    pub async fn update_clipboard(&self, app: &AppHandle) -> Result<(), Error> {
        // get the current clip text
        let clipboard_clip_text = clip_data_from_system_clipboard(app)?;
        // if the current clip text is empty, then return
        if clipboard_clip_text.is_empty() {
            debug!("The clipboard text is empty, do not create a new clip");
            return Ok(());
        }

        // get the current clip text
        let current_clip = self.get_current_clip().await?;
        let current_clip_text = if let Some(current_clip_text) = current_clip {
            current_clip_text.get_text()
        } else {
            Arc::new("".to_string())
        };

        // if the current clip text is the same as the current clip text, then return
        if *clipboard_clip_text == *current_clip_text {
            debug!(
                "The clipboard text is the same as the current clip text, do not create a new clip"
            );
            return Ok(());
        }

        // if the current clip text is the same as the last clip text, then return
        let last_clip_mutex = self.clipboard_monitor_last_clip.lock().await;
        let last_clip = last_clip_mutex.clone();
        drop(last_clip_mutex);
        if *clipboard_clip_text == *last_clip {
            debug!(
                "The clipboard text is the same as the last clip text, do not create a new clip"
            );
            return Ok(());
        }

        let id = self.new_clip(clipboard_clip_text.clone(), app).await?;
        let mut last_clip = self.clipboard_monitor_last_clip.lock().await;
        *last_clip = clipboard_clip_text;
        drop(last_clip);
        let mut clips = self.clips.lock().await;
        clips.current_clip = ClipID::Clip(id);
        drop(clips);

        // update the tray
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::TrayUpdateEvent).await;

        Ok(())
    }

    /// Get the current page
    ///
    /// Will try lock `clips`.
    pub async fn get_current_page(&self) -> usize {
        let clips = self.clips.lock().await;
        clips.current_page
    }

    /// Set the current page
    ///
    /// Will try lock `clips`.
    pub async fn set_current_page(&self, page: usize) {
        let mut clips = self.clips.lock().await;
        clips.current_page = page;
    }

    /// Get the len of whole_list_of_ids
    ///
    /// Will try lock `clips`.
    pub async fn get_whole_list_of_ids_len(&self) -> usize {
        let clips = self.clips.lock().await;
        clips.whole_list_of_ids.len()
    }

    /// Update the normal clip tray
    ///
    /// Only used in `self.update_tray()`
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    async fn update_normal_clip_tray(
        &self,
        app: &AppHandle,
        clips_per_page: i64,
        whole_list_of_ids_len: usize,
        max_clip_length: i64,
        current_page: i64,
    ) -> Result<(), Error> {
        // get the current page clips
        let mut current_page_clips: Vec<ClipType> = Vec::new();
        for i in 0..clips_per_page {
            let pos = whole_list_of_ids_len as i64 - (current_page * clips_per_page + i + 1);
            if pos < 0 {
                break;
            }
            let clips = self.clips.lock().await;
            let clip_id = clips.whole_list_of_ids.get(pos as usize);
            if clip_id.is_none() {
                break;
            }
            let clip_id: i64 = *clip_id.unwrap();
            drop(clips);
            let clip = self.get_clip(ClipID::Clip(clip_id)).await?;
            if clip.is_none() {
                break;
            }
            current_page_clips.push(clip.unwrap());
        }

        // put text in
        // empty the tray_ids_map
        let mut clips = self.clips.lock().await;
        clips.tray_ids_map.clear();
        drop(clips);
        for i in 0..current_page_clips.len() {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);

            let c = current_page_clips.get(i);
            if c.is_none() {
                continue;
            }
            let c = c.unwrap();

            let text = c.get_text();
            let text = trim_clip_text(text, max_clip_length);
            let res = tray_clip_sub_menu.set_title(text);
            if res.is_err() {
                return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
            }
            let mut clips = self.clips.lock().await;
            clips.tray_ids_map.push(c.get_id());
            drop(clips);
        }

        // clean out the rest of the tray
        for i in current_page_clips.len()..clips_per_page as usize {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);
            let res = tray_clip_sub_menu.set_title("".to_string());
            if res.is_err() {
                return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
            }
        }

        Ok(())
    }

    /// Update the tray page info
    ///
    /// Only used in `self.update_tray()`
    /// '''
    /// Total clips: {}, Current page: {}/{}
    /// '''
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    async fn update_tray_page_info(
        &self,
        app: &AppHandle,
        whole_list_of_ids_len: usize,
        current_page: i64,
        whole_pages: usize,
    ) -> Result<(), Error> {
        let tray_page_info_item = app.tray_handle().get_item("page_info");
        let tray_page_info_title = format!(
            "Total clips: {}, Current page: {}/{}",
            whole_list_of_ids_len,
            current_page + 1,
            whole_pages + 1
        );
        let res = tray_page_info_item.set_title(tray_page_info_title);
        if res.is_err() {
            return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
        }

        Ok(())
    }

    /// Update the pinned clips in the tray
    ///
    /// Only used in `self.update_tray()`
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    async fn update_pinned_clips(
        &self,
        app: &AppHandle,
        max_clip_length: i64,
    ) -> Result<(), Error> {
        let clips = self.clips.lock().await;
        let pinned_clips_len = clips.pinned_clips.len();
        drop(clips);
        // update pinned clips in the tray
        for i in 0..pinned_clips_len {
            let clips = self.clips.lock().await;
            let pinned_clip = clips.pinned_clips.get(i);
            if pinned_clip.is_none() {
                continue;
            }
            let pinned_clip = pinned_clip.unwrap();
            let pinned_clip_text = pinned_clip.text.clone();
            drop(clips);
            let pinned_clip_text = trim_clip_text(pinned_clip_text, max_clip_length);
            let pinned_clip_id = "pinned_clip_".to_string() + &i.to_string();
            let pinned_clip_item: tauri::SystemTrayMenuItemHandle =
                app.tray_handle().get_item(&pinned_clip_id);
            let res = pinned_clip_item.set_title(pinned_clip_text);
            if res.is_err() {
                return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
            }
        }

        Ok(())
    }

    /// Update the tray with all the current clips and pinned clips, and other data
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    #[warn(unused_must_use)]
    pub async fn update_tray(&self, app: &AppHandle) -> Result<(), Error> {
        // get the clips per page configuration
        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clips_per_page = config.clip_per_page;
        let max_clip_length = config.clip_max_show_length;
        drop(config);

        // get the current page
        let mut current_page = self.get_current_page().await;
        let whole_pages = self.get_max_page(app).await;
        let whole_list_of_ids_len = self.get_whole_list_of_ids_len().await;
        // if the current page bigger than the whole pages, then calculate the current page, regarding to current_clip_pos
        if current_page > whole_pages {
            // get the current clip pos
            let current_clip_pos = self.get_current_clip_pos().await;
            let current_clip_pos = if let Some(current_clip_pos) = current_clip_pos {
                current_clip_pos
            } else if whole_list_of_ids_len == 0 {
                0
            } else {
                whole_list_of_ids_len - 1
            };

            if whole_list_of_ids_len == current_clip_pos {
                current_page = 0;
            } else {
                current_page =
                    (whole_list_of_ids_len - current_clip_pos - 1) / clips_per_page as usize;
            }
            self.set_current_page(current_page).await;
        }

        let update_page_info_res = self.update_tray_page_info(
            app,
            whole_list_of_ids_len,
            current_page as i64,
            whole_pages,
        );
        let update_pinned_res = self.update_pinned_clips(app, max_clip_length);
        let update_normal_res = self.update_normal_clip_tray(
            app,
            clips_per_page,
            whole_list_of_ids_len,
            max_clip_length,
            current_page as i64,
        );

        update_page_info_res.await?;
        update_pinned_res.await?;
        update_normal_res.await?;

        Ok(())
    }

    /// Get the position of the pinned clip in the pinned clips with text.
    ///
    /// If the clip is not in the pinned clips, then return None.
    ///
    /// Will try lock `clips`.
    ///
    /// As text do not have any order, we have to do a linear search.
    async fn get_pos_of_pinned_clips_text(&self, text: Arc<String>) -> Option<usize> {
        let clips = self.clips.lock().await;
        for (i, pinned_clip) in clips.pinned_clips.iter().enumerate() {
            if pinned_clip.text == text {
                return Some(i);
            }
        }

        None
    }

    /// New a pinned clip in the database and return the id of the new pinned clip
    ///     - insert the pinned clip to the database
    ///     - insert the pinned clip to the pinned clips
    ///     - will not update the tray
    ///
    /// Will try lock `database_connection` and `clips`.
    pub async fn add_pinned_clips(&self, text: Arc<String>) -> Result<(), Error> {
        debug!("Add clip to pinned clips");
        // if the clip is already in the pinned clips, then return
        let text1 = text.clone();
        if self.get_pos_of_pinned_clips_text(text1).await.is_some() {
            debug!("Clip is already in pinned clips, not adding it");
            return Ok(());
        }

        // if the clip is not in the pinned clips, then add it to the database pinned_clips
        let timestamp = get_system_timestamp();

        debug!("Insert clip into database");
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let text1 = text.clone();
        let res = sqlx::query("INSERT INTO pinned_clips (text, timestamp) VALUES (?, ?)")
            .bind(&(*text))
            .bind(timestamp)
            .fetch_optional(db_connection.as_ref())
            .await;
        let text2 = text1.clone();
        if let Err(err) = res {
            return Err(Error::InsertClipIntoDatabaseErr(
                format!("insert clip text: {} into pinned clips", text1),
                err.to_string(),
            ));
        }

        // get the id of the clip
        let res = sqlx::query("SELECT id FROM pinned_clips ORDER BY id DESC LIMIT 1")
            .fetch_optional(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::GetClipDataFromDatabaseErr(-1, err.to_string()));
        }

        if let Some(row) = res.unwrap() {
            let id: i64 = row.get("id");

            let pinned_clip = PinnedClip {
                id,
                text: text2,
                timestamp,
            };

            debug!("Add clip to pinned clips");
            let mut clips = self.clips.lock().await;
            clips.pinned_clips.push(pinned_clip);
        } else {
            return Err(Error::GetClipDataFromDatabaseErr(
                -1,
                "no pinned clip obtained, but we expected to have inserted one".to_string(),
            ));
        }

        Ok(())
    }

    /// Remove a pinned clip from the database and the pinned clips
    ///     - remove the pinned clip from the database
    ///     - remove the pinned clip from the pinned clips
    ///     - will not update the tray
    ///
    /// Will try lock `database_connection` and `clips`.
    pub async fn remove_pinned_clip(&self, text: Arc<String>) -> Result<(), Error> {
        let text1 = text.clone();
        let index = self.get_pos_of_pinned_clips_text(text1).await;
        // if the clip is not in the pinned clips, then return
        if index.is_none() {
            return Ok(());
        }

        // if the clip is in the pinned clips, then remove it from the pinned clips
        let mut clips = self.clips.lock().await;
        clips.pinned_clips.remove(index.unwrap() as usize);
        drop(clips);

        // remove the clip from database
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query("DELETE FROM pinned_clips WHERE text = ?")
            .bind(&(*text))
            .fetch_optional(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::DeleteClipFromDatabaseErr(-1, err.to_string()));
        }

        Ok(())
    }
}

/// The current clip text in the system clipboard
fn clip_data_from_system_clipboard(app: &AppHandle) -> Result<Arc<String>, Error> {
    let clipboard_manager = app.clipboard_manager();
    let clip = clipboard_manager.read_text();
    if let Err(err) = clip {
        return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
    }
    let clip = clip.unwrap();
    let clip = if let Some(res) = clip {
        res
    } else {
        "".to_string()
    };

    Ok(Arc::new(clip))
}

/// chars that consider as white space
static WHITE_SPACE: Lazy<Vec<&str>> = Lazy::new(|| vec![" ", "\t", "\n", "\r"]);

/// Trim the text to the given length.
///
/// Also take care of slicing the text in the middle of a unicode character
/// Also take care of the width of a unicode character
///
/// l is treated as 20 if l <= 6
fn trim_clip_text(text: Arc<String>, l: i64) -> String {
    // trim the leading white space
    let mut text = text.graphemes(true);
    let l = if l <= 6 { 20 } else { l };

    let mut res: String = String::new();
    loop {
        let char = text.next();
        if char.is_none() {
            break;
        }
        let char = char.unwrap();
        if WHITE_SPACE.contains(&char) {
            continue;
        } else {
            res += char;
            break;
        }
    }

    let mut final_width = 0;
    loop {
        let char = text.next();
        if char.is_none() {
            break;
        }
        let char = char.unwrap();
        let width = unicode_width::UnicodeWidthStr::width(char);
        if final_width + width > l as usize {
            res.push_str("...");
            break;
        }
        final_width += width;
        res.push_str(char);
    }

    res
}
