use crate::{
    clip::get_system_timestamp,
    config::ConfigMutex,
    error::Error,
    event::{CopyClipEvent, EventSender},
};
use std::sync::Arc;

use super::{cache::CACHED_CLIP, clip_struct::Clip};
use log::debug;
use once_cell::sync::Lazy;
use sqlx::{Row, SqlitePool};
use tauri::{async_runtime::Mutex, AppHandle, ClipboardManager, Manager};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default, Clone)]
pub struct Clips {
    /// the id of the current clip
    pub current_clip: Option<i64>,
    /// the current page
    pub current_page: usize,
    /// the ids of all the clips, well sorted               
    pub whole_list_of_ids: Vec<i64>,
    /// the ids of the current displayed clips, the same order with the order displayed in the tray     
    pub tray_ids_map: Vec<i64>,
    /// the pinned clips
    pub pinned_clips: Vec<i64>,
    /// the favourite clips
    pub favourite_clips: Vec<i64>,
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
    ///     - edit the clips.favourite_clips to add or remove the id
    ///
    /// Will try lock `database_connection`.
    pub async fn change_clip_favourite_status(&self, id: i64, target: bool) -> Result<(), Error> {
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

        // edit the favourite clips slot in the tray
        if target {
            // decide whether the id is in the favourite_clips vector list
            let mut clips = self.clips.lock().await;
            let res = clips.favourite_clips.binary_search(&id);
            if res.is_err() {
                // if the id is not in the favourite_clips vector list, then add it to the list
                clips.favourite_clips.push(id);
                // sort
                clips.favourite_clips.sort();
            }
        } else {
            // decide whether the id is in the favourite_clips vector list
            let mut clips = self.clips.lock().await;
            let res = clips.favourite_clips.binary_search(&id);
            if let Ok(pos) = res {
                // if the id is in the favourite_clips vector list, then remove it from the list
                clips.favourite_clips.remove(pos);
            }
        }

        Ok(())
    }

    /// change the clip pinned state to the target state
    ///     - change the clip in the cache
    ///     - change the clip in the database
    ///     - edit the clips.pinned_clips to add or remove the id
    ///
    /// Will try lock `database_connection` and `clips`.
    pub async fn change_clip_pinned_status(&self, id: i64, target: bool) -> Result<(), Error> {
        // change the clip in the cache
        // no need to wait for the result
        CACHED_CLIP.update_pinned_state(id, target).await;

        // change the clip in the database
        let db_connection_mutex = self.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query("UPDATE clips SET pinned = ? WHERE id = ?")
            .bind(target)
            .bind(id)
            .fetch_optional(db_connection.as_ref())
            .await;
        if let Err(err) = res {
            return Err(Error::UpdateClipsInDatabaseErr(
                format!(
                    "toggle clip pinned state of id: {} to pinned state:{}",
                    id, target
                ),
                err.to_string(),
            ));
        }

        // edit the pinned clips slot in the tray
        if target {
            // decide whether the id is in the pinned_clips vector list
            let mut clips = self.clips.lock().await;
            let res = clips.pinned_clips.binary_search(&id);
            if res.is_err() {
                // if the id is not in the pinned_clips vector list, then add it to the list
                clips.pinned_clips.push(id);
                // sort
                clips.pinned_clips.sort();
            }
        } else {
            // decide whether the id is in the pinned_clips vector list
            let mut clips = self.clips.lock().await;
            let res = clips.pinned_clips.binary_search(&id);
            if let Ok(pos) = res {
                // if the id is in the pinned_clips vector list, then remove it from the list
                clips.pinned_clips.remove(pos);
            }
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
    pub async fn delete_clip(&self, mut id: Option<i64>) -> Result<(), Error> {
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
        // get the position of the id in the whole list of ids
        // return if current clip is pinned clip
        let mut clips = self.clips.lock().await;
        if let Some(id_c) = clips.current_clip {
            if id_c == id {
                clips.current_clip = None;
            }
        }
        let whole_list_index = clips.whole_list_of_ids.binary_search(&id);
        if let Ok(whole_list_index) = whole_list_index {
            clips.whole_list_of_ids.remove(whole_list_index);
        }

        // delete from the pinned_clips list
        let pinned_list_index = clips.pinned_clips.binary_search(&id);
        if let Ok(pinned_list_index) = pinned_list_index {
            clips.pinned_clips.remove(pinned_list_index);
        }

        // delete from the favourite_clips list
        let favourite_list_index = clips.favourite_clips.binary_search(&id);
        if let Ok(favourite_list_index) = favourite_list_index {
            clips.favourite_clips.remove(favourite_list_index);
        }

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

    /// Get a clip by id
    ///
    /// Get the latest clip if the id is none.
    /// None if the clip is not found.
    ///
    /// Will try lock `database_connection` and `clips`.
    #[warn(unused_must_use)]
    pub async fn get_clip(&self, mut id: Option<i64>) -> Result<Option<Clip>, Error> {
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

        let pinned = res.try_get::<i64, _>("pinned");
        if let Err(err) = pinned {
            return Err(Error::GetClipDataFromDatabaseErr(id, err.to_string()));
        }
        let pinned = pinned.unwrap() == 1;

        let clip = Clip {
            text: Arc::new(text.unwrap()),
            timestamp: timestamp.unwrap(),
            id,
            favourite,
            pinned,
        };

        // add the clip to the cache
        CACHED_CLIP.insert(id, clip.clone()).await;

        Ok(clip.into())
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
    pub async fn get_current_clip(&self) -> Result<Option<Clip>, Error> {
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

        current?;

        self.get_clip_pos(current).await
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

        let timestamp = get_system_timestamp();

        let db_connection_mutex = app.state::<ClipData>();
        let db_connection_mutex = db_connection_mutex.database_connection.lock().await;
        let db_connection = db_connection_mutex.clone().unwrap();
        drop(db_connection_mutex);
        let res = sqlx::query(
            "INSERT INTO clips (id, text, timestamp, favourite, pinned) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&(*text))
        .bind(timestamp)
        .bind(0)
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
            pinned: false,
        };

        // add the clip to the cache
        CACHED_CLIP.insert(id, clip).await;

        let mut clips = self.clips.lock().await;
        clips.whole_list_of_ids.push(id);

        // change the current clip to the last one
        clips.current_clip = Some(id);

        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        if !config.auto_delete_duplicate_clip {
            debug!("Auto delete duplicate clip is disabled");
            return Ok(id);
        }
        drop(config);

        debug!("Start auto delete duplicate clip");
        let res = sqlx::query("SELECT id, favourite, pinned FROM clips WHERE text = ?")
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
        let mut ids_to_delete: Vec<i64> = Vec::new();
        let c = clips.current_clip;
        drop(clips);
        let mut is_favourite = false;
        let mut is_pinned = false;
        for row in res {
            let id = row.try_get("id");
            if let Ok(id) = id {
                if Some(id) == c {
                    continue;
                }
                ids_to_delete.push(id);
            }
            let favourite: Result<i64, sqlx::Error> = row.try_get("favourite");
            if let Ok(favourite) = favourite {
                if favourite == 1 {
                    is_favourite = true;
                }
            }
            let pinned: Result<i64, sqlx::Error> = row.try_get("pinned");
            if let Ok(pinned) = pinned {
                if pinned == 1 {
                    is_pinned = true;
                }
            }
        }
        if is_favourite {
            self.change_clip_favourite_status(id, true).await?;
        }
        if is_pinned {
            self.change_clip_pinned_status(id, true).await?;
        }
        for id in ids_to_delete {
            debug!("Delete duplicate clip id: {}", id);
            self.delete_clip(Some(id)).await?;
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
    /// If id is None, then change to the latest clip.
    ///
    /// Will try `lock database_connection` and `clips`.
    #[warn(unused_must_use)]
    pub async fn select_clip(&self, app: &AppHandle, id: Option<i64>) -> Result<(), Error> {
        // try get the clip
        let c = self.get_clip(id).await?;
        if c.is_none() {
            debug!("The requested ClipID: {:?} is not found", id);
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
            current_clip_text.text
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
        clips.current_clip = Some(id);
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
        let mut current_page_clips: Vec<Clip> = Vec::new();
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
            let clip = self.get_clip(Some(clip_id)).await?;
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

            let text = c.text.clone();
            let text = trim_clip_text(text, max_clip_length);
            let res = tray_clip_sub_menu.set_title(text);
            if res.is_err() {
                return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
            }
            let mut clips = self.clips.lock().await;
            clips.tray_ids_map.push(c.id);
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
            let pinned_clip = *pinned_clip;
            drop(clips);
            let pinned_clip = self.get_clip(Some(pinned_clip)).await?;
            if pinned_clip.is_none() {
                continue;
            }
            let pinned_clip = pinned_clip.unwrap();
            let pinned_clip_text = pinned_clip.text.clone();
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

    /// Update the favourite clips in the tray
    ///
    /// Only used in `self.update_tray()`
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    async fn update_favourite_clips(
        &self,
        app: &AppHandle,
        max_clip_length: i64,
    ) -> Result<(), Error> {
        let clips = self.clips.lock().await;
        let favourite_clips_len = clips.favourite_clips.len();
        drop(clips);
        // update favourite clips in the tray
        for i in 0..favourite_clips_len {
            let clips = self.clips.lock().await;
            let favourite_clip = clips.favourite_clips.get(i);
            if favourite_clip.is_none() {
                continue;
            }
            let favourite_clip = favourite_clip.unwrap();
            let favourite_clip = *favourite_clip;
            drop(clips);
            let favourite_clip = self.get_clip(Some(favourite_clip)).await?;
            if favourite_clip.is_none() {
                continue;
            }
            let favourite_clip = favourite_clip.unwrap();
            let favourite_clip_text = favourite_clip.text.clone();
            let favourite_clip_text = trim_clip_text(favourite_clip_text, max_clip_length);
            let favourite_clip_id = "favourite_clip_".to_string() + &i.to_string();
            let favourite_clip_item: tauri::SystemTrayMenuItemHandle =
                app.tray_handle().get_item(&favourite_clip_id);
            let res = favourite_clip_item.set_title(favourite_clip_text);
            if res.is_err() {
                return Err(Error::SetSystemTrayTitleErr(res.err().unwrap().to_string()));
            }
        }

        Ok(())
    }

    /// Update the tray with all the current clips, the pinned clips, the favourite clips,
    /// and other data
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
        let update_favourite_res = self.update_favourite_clips(app, max_clip_length);

        update_page_info_res.await?;
        update_pinned_res.await?;
        update_normal_res.await?;
        update_favourite_res.await?;

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
