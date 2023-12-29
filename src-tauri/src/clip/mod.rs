pub mod database;
pub mod monitor;
pub mod pinned;
pub mod search;

use std::{cmp::Ordering, sync::Arc};

use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::{async_runtime::Mutex, AppHandle, ClipboardManager, Manager};

use crate::{
    config::ConfigMutex,
    error,
    event::{CopyClipEvent, EventSender},
};

static CACHED_CLIP: once_cell::sync::Lazy<moka::future::Cache<i64, Arc<std::sync::Mutex<Clip>>>> =
    once_cell::sync::Lazy::new(|| {
        moka::future::Cache::builder()
            .weigher(|_, value: &Arc<std::sync::Mutex<Clip>>| -> u32 {
                let clip = value.lock().unwrap();
                clip.text.len() as u32
            })
            .max_capacity(32 * 1024 * 1024)
            .initial_capacity(1000)
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build()
    });

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Clip {
    /// The text of the clip.
    /// After the clip is created, the text should not be changed
    #[serde(
        deserialize_with = "arc_string_deserialize",
        serialize_with = "arc_string_serialize"
    )]
    pub text: Arc<String>,
    /// in seconds
    pub timestamp: i64,
    /// the id of the clip
    pub id: i64,
    ///  if the clip is a favourite 1 means true, 0 means false
    pub favourite: bool,
}

fn arc_string_deserialize<'de, D>(deserializer: D) -> Result<Arc<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(Arc::new(s))
}

fn arc_string_serialize<S>(s: &Arc<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(s)
}

impl Clip {
    /// copy the clip to the clipboard
    pub fn copy_clip_to_clipboard(&self, app: &AppHandle) -> Result<(), error::Error> {
        let mut clipboard_manager = app.clipboard_manager();
        let res = clipboard_manager.write_text((*self.text).clone());
        if let Err(e) = res {
            return Err(error::Error::WriteToSystemClipboardErr(
                (*self.text).clone(),
                e.to_string(),
            ));
        }
        Ok(())
    }

    /// convert to json format string
    pub fn to_json_string(&self) -> Result<String, error::Error> {
        let res = serde_json::to_string(self);
        if let Err(e) = res {
            return Err(error::Error::ExportError(e.to_string()));
        }
        Ok(res.unwrap())
    }
}

#[derive(Debug)]
pub struct Clips {
    /// the id of the current clip
    pub current_clip: i64,
    /// the current page           
    pub current_page: i64,
    /// the ids of all the clips, well sorted               
    pub whole_list_of_ids: Vec<i64>,
    /// the ids of the current displaying clips, the same order with the order displaying in the tray     
    pub tray_ids_map: Vec<i64>,
    /// the ids of the pinned clips
    pub pinned_clips_ids: Vec<i64>,
}

impl Default for Clips {
    fn default() -> Self {
        Self {
            current_clip: -1,
            current_page: 0,
            whole_list_of_ids: Default::default(),
            tray_ids_map: Default::default(),
            pinned_clips_ids: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ClipData {
    /// the clips data
    pub clips: Clips,
    /// the database connection
    database_connection: Option<SqlitePool>,
    /// monitor the clipboard
    clipboard_monitor: ClipboardMonitor,
}

#[derive(Debug, Default)]
pub struct ClipboardMonitor {
    /// the last clip
    last_clip: Arc<String>,
}

pub struct ClipDataMutex {
    pub clip_data: Mutex<ClipData>,
}

impl ClipData {
    /// change the clip favourite state to the target state
    pub async fn change_favourite_clip(
        &mut self,
        id: i64,
        target: bool,
    ) -> Result<(), error::Error> {
        // change the clip favourite state to the target state
        // change the clip in the cache
        // change the clip in the database

        // change the clip in the cache
        let clip = CACHED_CLIP.get(&id);
        if let Some(clip) = clip {
            let mut clip = clip.lock().unwrap();
            clip.favourite = target;
        }

        // change the clip in the database
        let res = sqlx::query("UPDATE clips SET favourite = ? WHERE id = ?")
            .bind(target)
            .bind(id)
            .fetch_optional(self.database_connection.as_ref().unwrap())
            .await;

        if let Err(err) = res {
            return Err(error::Error::UpdateClipsInDatabaseErr(
                format!(
                    "toggle clip favourite state of id: {} to favourite state:{}",
                    id, target
                ),
                err.to_string(),
            ));
        }

        Ok(())
    }

    pub async fn delete_clip(&mut self, id: i64) -> Result<(), error::Error> {
        // delete a clip from the database and the cache

        // first delete in cache
        CACHED_CLIP.invalidate(&id).await;

        // delete in database
        let res = sqlx::query("DELETE FROM clips WHERE id = ?")
            .bind(id)
            .fetch_optional(self.database_connection.as_ref().unwrap())
            .await;
        if let Err(err) = res {
            return Err(error::Error::DeleteClipFromDatabaseErr(id, err.to_string()));
        }

        // delete from the whole list of ids
        // get the position of the id in the whole list of ids
        let pos = self.get_id_pos_in_whole_list_of_ids(id);
        let current_clip_pos = self.get_id_pos_in_whole_list_of_ids(self.clips.current_clip);
        if let Some(pos) = pos {
            // if pos is before the current clip, decrease the current clip by 1
            // if pos is the current clip, set the current clip to -1
            if let Some(current_clip_pos) = current_clip_pos {
                match pos.cmp(&current_clip_pos) {
                    Ordering::Less => {
                        self.clips.current_clip = *self
                            .clips
                            .whole_list_of_ids
                            .get(current_clip_pos as usize - 1)
                            .unwrap();
                    }
                    Ordering::Equal => {
                        self.clips.current_clip = -1;
                    }
                    Ordering::Greater => {}
                }
            } else {
                self.clips.current_clip = -1;
            }
            self.clips.whole_list_of_ids.remove(pos.try_into().unwrap());
        }
        Ok(())
    }

    pub fn first_page(&mut self) {
        // switch to the first page

        self.clips.current_page = 0;
    }

    /// get the clip with id
    /// if the given id is -1, it will return the latest clip,
    /// if no clip with id exist, it will try return an error dw
    #[warn(unused_must_use)]
    pub async fn get_clip(
        &mut self,
        mut id: i64,
    ) -> Result<Arc<std::sync::Mutex<Clip>>, error::Error> {
        // if id is -1, change it to the latest clip
        if id == -1 {
            if self.clips.whole_list_of_ids.is_empty() {
                return Err(error::Error::WholeListIDSEmptyErr);
            }
            let t = self.clips.whole_list_of_ids.last();
            if t.is_none() {
                return Err(error::Error::InvalidIDFromWholeListErr(None));
            }
            let t = t.unwrap();
            id = *t;
            if id < 0 {
                return Err(error::Error::InvalidIDFromWholeListErr(Some(id)));
            }
        }

        // if the clip is in the cache, return it
        let clip = CACHED_CLIP.get(&id);
        if let Some(clip) = clip {
            return Ok(clip);
        }

        // if the clip is not in the cache, get it from the database
        let res = sqlx::query("SELECT * FROM clips WHERE id = ?")
            .bind(id)
            .fetch_optional(self.database_connection.as_ref().unwrap())
            .await;
        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }
        let res = res.unwrap();

        if res.is_none() {
            return Err(error::Error::ClipNotFoundErr(id));
        }
        let res = res.unwrap();

        let text = res.try_get::<String, _>("text");
        if let Err(err) = text {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }

        let timestamp = res.try_get::<i64, _>("timestamp");
        if let Err(err) = timestamp {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }

        let id_new = res.try_get::<i64, _>("id");
        if let Err(err) = id_new {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }
        let id = id_new.unwrap();

        let favourite = res.try_get::<i64, _>("favourite");
        if let Err(err) = favourite {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }
        let favourite = favourite.unwrap() == 1;

        let clip = Clip {
            text: Arc::new(text.unwrap()),
            timestamp: timestamp.unwrap(),
            id,
            favourite,
        };

        let clip = Arc::new(std::sync::Mutex::new(clip));

        // add the clip to the cache
        CACHED_CLIP.insert(id, clip.clone()).await;

        Ok(clip)
    }

    /// get the position of the clip in the whole list of ids
    /// return the position of the clip in the whole list of ids
    pub fn get_clip_pos(&self, id: i64) -> i64 {
        // get the position of the clip in the whole_list_of_ids
        // if the id is not in the list, return the highest pos
        let clip_pos = self.get_id_pos_in_whole_list_of_ids(id);
        if clip_pos.is_none() {
            return self.clips.whole_list_of_ids.len() as i64 - 1;
        }

        clip_pos.unwrap()
    }

    pub async fn get_current_clip(&mut self) -> Result<Arc<std::sync::Mutex<Clip>>, error::Error> {
        self.get_clip(self.clips.current_clip).await
    }

    pub fn get_current_clip_pos(&self) -> i64 {
        self.get_clip_pos(self.clips.current_clip)
    }

    /// get the position of the id in the whole list of ids
    /// if the id is not in the list, return None
    /// use binary search
    pub fn get_id_pos_in_whole_list_of_ids(&self, id: i64) -> Option<i64> {
        let pos = self.clips.whole_list_of_ids.binary_search(&id);
        if pos.is_err() {
            return None;
        }

        Some(pos.unwrap() as i64)
    }

    /// get the maximum page number depend on the number of clips and the number of clips per page
    pub async fn get_max_page(&self, app: &AppHandle) -> i64 {
        // get the max page number
        // if there is no limit, return -1

        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let max_page = self.clips.whole_list_of_ids.len() as i64 / config.clip_per_page;
        if max_page * config.clip_per_page == self.clips.whole_list_of_ids.len() as i64 {
            return max_page - 1;
        }
        max_page
    }

    /// create a new clip in the database and return the id of the new clip
    pub async fn new_clip(
        &mut self,
        text: Arc<String>,
        app: &AppHandle,
    ) -> Result<i64, error::Error> {
        let id: i64 = if let Some(id) = self.clips.whole_list_of_ids.last() {
            *id + 1
        } else {
            1
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let res =
            sqlx::query("INSERT INTO clips (id, text, timestamp, favourite) VALUES (?, ?, ?, ?)")
                .bind(id)
                .bind(&(*text))
                .bind(timestamp)
                .bind(0)
                .fetch_optional(self.database_connection.as_ref().unwrap())
                .await;
        if let Err(err) = res {
            return Err(error::Error::InsertClipIntoDatabaseErr(
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
        CACHED_CLIP
            .insert(id, Arc::new(std::sync::Mutex::new(clip)))
            .await;

        self.clips.whole_list_of_ids.push(id);

        // change the current clip to the last one
        self.clips.current_clip = id;

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
            .fetch_all(self.database_connection.as_ref().unwrap())
            .await;
        if let Err(err) = res {
            return Err(error::Error::GetClipDataFromDatabaseErr(
                id,
                err.to_string(),
            ));
        }
        let res = res.unwrap();
        if res.len() <= 1 {
            return Ok(id);
        }
        let mut ids_to_delete = Vec::new();
        for row in res {
            let id: i64 = row.try_get("id").unwrap();
            if id == self.clips.current_clip {
                continue;
            }
            ids_to_delete.push(id);
        }
        for id in ids_to_delete {
            debug!("Delete duplicate clip id: {}", id);
            self.delete_clip(id).await?;
        }

        Ok(id)
    }

    pub async fn next_page(&mut self, app: &AppHandle) {
        // switch to the next page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await;
        self.switch_page(1, max_page);
    }

    pub async fn prev_page(&mut self, app: &AppHandle) {
        // switch to the previous page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await;
        self.switch_page(-1, max_page);
    }

    /// change the clipboard to the selected clip
    /// and also change the current clip to the selected clip
    #[warn(unused_must_use)]
    pub async fn select_clip(&mut self, app: &AppHandle, id: i64) -> Result<(), error::Error> {
        // select a clip by id
        // change the current clip to the clip, and fill in the text of the clip to the clipboard

        // try get the clip
        let c = self.get_clip(id).await;
        if c.is_err() {
            return Err(c.err().unwrap());
        }

        let c = c.unwrap();
        self.clips.current_clip = id;

        let c = c.lock().unwrap();
        c.copy_clip_to_clipboard(app)?;

        Ok(())
    }

    /// switch the tray + number of page, if the page is not in the range, then do nothing
    pub fn switch_page(&mut self, page: i64, max_page: i64) {
        // switch to the page by the page number
        // if max_page is -1, it means there is no limit

        let target_page = self.clips.current_page + page;
        if target_page < 0 {
            self.clips.current_page = 0;
            return;
        }
        if target_page > max_page && max_page >= 0 {
            self.clips.current_page = max_page;
            return;
        }

        self.clips.current_page = target_page;
    }

    /// handle the clip board change event
    #[warn(unused_must_use)]
    pub async fn update_clipboard(&mut self, app: &AppHandle) -> Result<(), error::Error> {
        // get the current clip text
        let current_clip = self.get_current_clip().await;
        let current_clip_text = match current_clip {
            Err(error::Error::ClipNotFoundErr(_)) => Arc::new(String::new()),
            Err(error::Error::WholeListIDSEmptyErr) => Arc::new(String::new()),
            Err(err) => {
                return Err(err);
            }
            Ok(clip) => {
                let c = clip.lock().unwrap();
                c.text.clone()
            }
        };

        // get the current clip text
        let clipboard_clip_text = clip_data_from_system_clipbaord(app)?;

        // if the current clip text is empty, then return
        if clipboard_clip_text.is_empty() {
            return Ok(());
        }

        // if the current clip text is the same as the current clip text, then return
        if clipboard_clip_text == *current_clip_text {
            return Ok(());
        }

        // if the current clip text is the same as the last clip text, then return
        if clipboard_clip_text == *self.clipboard_monitor.last_clip {
            return Ok(());
        }

        let clipboard_clip_text = Arc::new(clipboard_clip_text);

        let id = self.new_clip(clipboard_clip_text.clone(), app).await?;
        self.clipboard_monitor.last_clip = clipboard_clip_text;
        self.clips.current_clip = id;

        // update the tray
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::TrayUpdateEvent);

        Ok(())
    }

    /// update the tray to the current clip and also current page
    #[warn(unused_must_use)]
    pub async fn update_tray(&mut self, app: &AppHandle) -> Result<(), error::Error> {
        // get the clips per page configuration
        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clips_per_page = config.clip_per_page;
        let max_clip_length = config.clip_max_show_length;
        drop(config);

        // get the current page
        let mut current_page = self.clips.current_page;
        let whole_pages = self.get_max_page(app).await;
        // if the current page bigger than the whole pages, then calculate the current page, regarding to current_clip_pos
        if current_page > whole_pages {
            // get the current clip pos
            let current_clip_pos: i64 = self.get_current_clip_pos();

            current_page =
                (self.clips.whole_list_of_ids.len() as i64 - current_clip_pos - 1) / clips_per_page;
            self.clips.current_page = current_page;
        }

        // get the current page clips
        let mut current_page_clips = Vec::new();
        for i in 0..clips_per_page {
            let pos =
                self.clips.whole_list_of_ids.len() as i64 - (current_page * clips_per_page + i + 1);
            if pos < 0 {
                break;
            }
            let clip_id = self.clips.whole_list_of_ids.get(pos as usize);
            if clip_id.is_none() {
                break;
            }
            let clip_id = clip_id.unwrap();
            let clip = self.get_clip(*clip_id).await?;
            current_page_clips.push(clip);
        }

        // put text in
        // empty the tray_ids_map
        self.clips.tray_ids_map.clear();
        for i in 0..current_page_clips.len() {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);

            let c = current_page_clips.get(i);
            if c.is_none() {
                continue;
            }
            let c = c.unwrap();
            let c = c.lock().unwrap();

            let text = c.text.clone();
            let text = trim_clip_text(text, max_clip_length);
            let res = tray_clip_sub_menu.set_title(text);
            if res.is_err() {
                return Err(error::Error::SetSystemTrayTitleErr(
                    res.err().unwrap().to_string(),
                ));
            }
            self.clips.tray_ids_map.push(c.id)
        }

        // clean out the rest of the tray
        for i in current_page_clips.len()..clips_per_page as usize {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);
            let res = tray_clip_sub_menu.set_title("".to_string());
            if res.is_err() {
                return Err(error::Error::SetSystemTrayTitleErr(
                    res.err().unwrap().to_string(),
                ));
            }
        }

        let tray_page_info_item = app.tray_handle().get_item("page_info");
        let tray_page_info_title = format!(
            "Total clips: {}, Current page: {}/{}",
            self.clips.whole_list_of_ids.len(),
            current_page + 1,
            whole_pages + 1
        );
        let res = tray_page_info_item.set_title(tray_page_info_title);
        if res.is_err() {
            return Err(error::Error::SetSystemTrayTitleErr(
                res.err().unwrap().to_string(),
            ));
        }

        // TODO update pinned clips
        for i in 0..self.clips.pinned_clips_ids.len() {
            let pinned_clip_id = self.clips.pinned_clips_ids.get(i);
            if pinned_clip_id.is_none() {
                continue;
            }
            let pinned_clip_id = pinned_clip_id.unwrap();
            let pinned_clip = self.get_clip(*pinned_clip_id).await?;
            let pinned_clip = pinned_clip.lock().unwrap();
            let pinned_clip_text = pinned_clip.text.clone();
            let pinned_clip_text = trim_clip_text(pinned_clip_text, max_clip_length);
            let pinned_clip_id = "pinned_clip_".to_string() + &i.to_string();
            let pinned_clip_item: tauri::SystemTrayMenuItemHandle =
                app.tray_handle().get_item(&pinned_clip_id);
            let res = pinned_clip_item.set_title(pinned_clip_text);
            if res.is_err() {
                return Err(error::Error::SetSystemTrayTitleErr(
                    res.err().unwrap().to_string(),
                ));
            }
        }

        Ok(())
    }

    /// add or remove a clip from the pinned clips
    ///
    /// if action is 0, then add the clip to the pinned clips
    /// if action is 1, then remove the clip from the pinned clips
    pub async fn add_remove_pinned_clip(
        &mut self,
        id: i64,
        action: i64,
    ) -> Result<(), error::Error> {
        // if action is 0, then add the clip to the pinned clips
        // if action is 1, then remove the clip from the pinned clips

        // if action is 0, then add the clip to the pinned clips
        if action == 0 {
            // if the clip is already in the pinned clips, then return
            if self.clips.pinned_clips_ids.contains(&id) {
                return Ok(());
            }

            // if the clip is not in the pinned clips, then add it to the pinned clips
            self.clips.pinned_clips_ids.push(id);

            // Add the clip from database
            let res = sqlx::query("INSERT INTO pinned_clips (id) VALUES (?)")
                .bind(id)
                .fetch_optional(self.database_connection.as_ref().unwrap())
                .await;
            if let Err(err) = res {
                return Err(error::Error::InsertClipIntoDatabaseErr(
                    format!("insert clip id: {} into pinned clips", id),
                    err.to_string(),
                ));
            }
        }

        // if action is 1, then remove the clip from the pinned clips
        if action == 1 {
            // if the clip is not in the pinned clips, then return
            if !self.clips.pinned_clips_ids.contains(&id) {
                return Ok(());
            }

            // if the clip is in the pinned clips, then remove it from the pinned clips
            let pos = self.clips.pinned_clips_ids.iter().position(|&r| r == id);
            if pos.is_none() {
                return Ok(());
            }
            let pos = pos.unwrap();
            self.clips.pinned_clips_ids.remove(pos);

            // remove the clip from database
            let res = sqlx::query("DELETE FROM pinned_clips WHERE id = ?")
                .bind(id)
                .fetch_optional(self.database_connection.as_ref().unwrap())
                .await;
            if let Err(err) = res {
                return Err(error::Error::DeleteClipFromDatabaseErr(id, err.to_string()));
            }
        }

        Ok(())
    }
}

/// Trim the text to the given length.
///
/// Also take care of slicing the text in the middle of a unicode character
fn trim_clip_text(text: Arc<String>, l: i64) -> String {
    // trim the leading white space
    let text = text.trim_start().to_string();

    if l <= 0 {
        return text;
    }
    if text.len() as i64 <= l {
        return text;
    }
    let mut res = String::new();
    if l <= 6 {
        for i in 0..l {
            res.push(text.chars().nth(i as usize).unwrap());
        }
        return res;
    }
    for i in 0..(l - 3) {
        let char = text.chars().nth(i as usize);
        if let Some(char) = char {
            res.push(char);
        } else {
            break;
        }
    }
    res.push_str("...");
    res
}

#[tauri::command]
pub async fn copy_clip_to_clipboard(
    app: tauri::AppHandle,
    event_sender: tauri::State<'_, EventSender>,
    clip_data: tauri::State<'_, ClipDataMutex>,
    id: i64,
) -> Result<(), String> {
    let mut clip_data = clip_data.clip_data.lock().await;
    let clip_data = clip_data.get_clip(id).await;
    if let Err(err) = clip_data {
        return Err(err.message());
    }
    let clip_data = clip_data.unwrap();
    let clip_data = clip_data.lock().unwrap();
    let res = clip_data.copy_clip_to_clipboard(&app);
    if let Err(err) = res {
        return Err(err.message());
    }

    event_sender.send(CopyClipEvent::SendNotificationEvent(
        "Clip copied to clipboard.".to_string(),
    ));

    Ok(())
}

#[tauri::command]
pub async fn delete_clip_from_database(
    clip_data: tauri::State<'_, ClipDataMutex>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
) -> Result<(), String> {
    let mut clip_data = clip_data.clip_data.lock().await;
    let res = clip_data.delete_clip(id).await;

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::TrayUpdateEvent);
    event_sender.send(CopyClipEvent::SendNotificationEvent(
        "Clip deleted from database.".to_string(),
    ));

    Ok(())
}

#[tauri::command]
pub async fn change_favourite_clip(
    clip_data: tauri::State<'_, ClipDataMutex>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
    target: bool,
) -> Result<(), String> {
    let mut clip_data = clip_data.clip_data.lock().await;
    let res = clip_data.change_favourite_clip(id, target).await;

    if let Err(err) = res {
        return Err(err.to_string());
    }

    event_sender.send(CopyClipEvent::SendNotificationEvent(
        "Clip favourite status changed.".to_string(),
    ));

    Ok(())
}

/// the function to handle the pinned clip change
/// if action is 0, then add the clip to the pinned clips
/// if action is 1, then remove the clip from the pinned clips
#[tauri::command]
pub async fn add_remove_pinned_clip(
    clip_data: tauri::State<'_, ClipDataMutex>,
    event_sender: tauri::State<'_, EventSender>,
    id: i64,
    action: i64,
) -> Result<(), String> {
    let mut clip_data = clip_data.clip_data.lock().await;
    let res = clip_data.add_remove_pinned_clip(id, action).await;

    if let Err(err) = res {
        return Err(err.to_string());
    }

    println!(
        "pinned clips ids: {:?}",
        clip_data.clips.pinned_clips_ids.len()
    );

    event_sender.send(CopyClipEvent::RebuildTrayMenuEvent);

    Ok(())
}

fn clip_data_from_system_clipbaord(app: &AppHandle) -> Result<String, error::Error> {
    let clipboard_manager = app.clipboard_manager();
    let clip = clipboard_manager.read_text();
    if let Err(err) = clip {
        return Err(error::Error::ReadFromSystemClipboardErr(err.to_string()));
    }
    let clip = clip.unwrap();
    let clip = if let Some(res) = clip {
        res
    } else {
        "".to_string()
    };

    Ok(clip)
}
