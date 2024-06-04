use std::path::PathBuf;

use crate::{
    config::ConfigMutex,
    database::DatabaseStateMutex,
    error::Error,
    event::{CopyClipEvent, EventSender},
};

use clip::{Clip, ClipType};
use log::debug;
use tauri::{async_runtime::Mutex, AppHandle, Manager};

use super::{copy_clip_to_clipboard_in, image_clip};

/// The clip data to be shared between threads
#[derive(Debug, Default, Clone)]
pub struct ClipState {
    /// the id of the current clip
    pub current_clip: Option<u64>,
    /// the current page
    pub current_page: u64,
}

/// The clip data to be shared between threads
#[derive(Debug, Default)]
pub struct ClipStateMutex {
    /// the clips data
    pub clip_state: Mutex<ClipState>,
}

impl ClipState {
    pub async fn get_total_number_of_clip(app: &AppHandle) -> Result<u64, anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .get_total_number_of_clip()
            .await
    }

    /// Trigger a tray update event
    pub async fn trigger_tray_update_event(&self, app: &AppHandle) {
        // trigger a tray update event
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;
    }

    /// Get the id of the clip with the position in current page
    ///
    /// If the position is out of range, return None.
    /// If the position is None, return the current clip id.
    pub async fn get_id_with_pos_in_current_page(
        &self,
        app: &AppHandle,
        pos: Option<u64>,
    ) -> Result<Option<u64>, anyhow::Error> {
        let current_page = self.current_page;

        // get page_len from the configuration file
        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clip_per_page: i64 = config.clip_per_page as i64;
        drop(config);

        // get total number of clips
        let total_number_of_clip: i64 = Self::get_total_number_of_clip(app).await? as i64;

        let pos = match pos {
            Some(pos) => {
                total_number_of_clip - (current_page as i64 * clip_per_page + pos as i64) - 1
            }
            None => match self.get_current_clip_pos(app).await? {
                Some(pos) => pos as i64,
                None => return Ok(None),
            },
        };

        if pos < 0 || pos >= total_number_of_clip {
            return Ok(None);
        }
        let pos = pos as u64;

        Self::get_id_with_pos(app, pos).await
    }

    /// Test if a clip have a label
    ///   - return true if the clip have the label
    ///   - otherwise return false
    pub async fn clip_have_label(
        app: &AppHandle,
        id: u64,
        label: &str,
    ) -> Result<bool, anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .clip_have_label(id, label)
            .await
    }

    /// Get the id of the latest clip,
    ///
    /// If there is no clip, return None.
    pub async fn get_latest_clip_id(app: &AppHandle) -> Result<Option<u64>, anyhow::Error> {
        app.state::<DatabaseStateMutex>().get_latest_clip_id().await
    }

    /// get how many clips one label has
    ///   - return the number of clips
    ///   - return 0 if the label is not found
    pub async fn get_label_clip_number(app: &AppHandle, label: &str) -> Result<u64, anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .get_label_clip_number(label)
            .await
    }

    /// get the clip with the label with the position of the clip in that label
    ///   - return the clip id
    ///   - return None if the clip is not found
    ///   - count from the lower id to the higher id
    ///   - for instance, label has 3 clips, then the first clip will have a position of 0
    pub async fn get_label_clip_id_with_pos(
        app: &AppHandle,
        label: &str,
        pos: u64,
    ) -> Result<Option<u64>, anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .get_label_clip_id_with_pos(label, pos)
            .await
    }

    /// add or delete a label to a clip
    ///   - if target is true, add the label to the clip
    ///   - if target is false, delete the label from the clip
    ///   - trigger a tray update event
    pub async fn change_clip_label(
        &mut self,
        app: &AppHandle,
        id: u64,
        label: &str,
        target: bool,
    ) -> Result<(), anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .change_clip_label(id, label, target)
            .await?;

        // trigger a tray update event
        self.trigger_tray_update_event(app).await;

        Ok(())
    }

    /// change the clip favourite state to the target state
    ///     - change the clip in the database
    ///     - trigger a tray update event
    pub async fn change_clip_favourite_status(
        &mut self,
        app: &AppHandle,
        id: u64,
        target: bool,
    ) -> Result<(), anyhow::Error> {
        self.change_clip_label(app, id, "favourite", target).await
    }

    /// change the clip pinned state to the target state
    ///     - change the clip in the cache
    ///     - change the clip in the database
    pub async fn change_clip_pinned_status(
        &mut self,
        app: &AppHandle,
        id: u64,
        target: bool,
    ) -> Result<(), anyhow::Error> {
        self.change_clip_label(app, id, "pinned", target).await
    }

    /// Delete a clip from the database and the cache,
    ///
    /// Will not trigger a tray update event.
    pub async fn delete_clip(
        &mut self,
        app: &AppHandle,
        id: Option<u64>,
    ) -> Result<(), anyhow::Error> {
        // if id is none, change it to the latest clip
        let id = match id {
            Some(id) => id,
            None => return Ok(()),
        };

        let c = self.get_clip(app, Some(id)).await?;
        let c = match c {
            Some(c) => c,
            None => return Ok(()),
        };

        // delete in database
        // wait for any error
        let db_connection = app.state::<DatabaseStateMutex>();
        let db_connection = db_connection.database_connection.lock().await;
        db_connection.execute("DELETE FROM clips WHERE id = ?", [id.to_string()])?;

        // if it is a image, decide whether it is the only image reference to that file
        if c.clip_type == ClipType::Image {
            match db_connection.query_row(
                "SELECT id FROM clips WHERE data = ?",
                [c.get_data_database()],
                |_| Ok(()),
            ) {
                Ok(_) => {
                    // there is still a clip reference to the image file
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    // there is not any clip reference to the image file
                    let path = PathBuf::from(c.decompress_text()?);
                    let thumbnail_path = clip::thumbnail_path(&path);
                    if path.exists() {
                        std::fs::remove_file(path)?;
                    };
                    if thumbnail_path.exists() {
                        std::fs::remove_file(thumbnail_path)?;
                    }
                }
                Err(err) => return Err(err.into()),
            };
        }

        self.trigger_tray_update_event(app).await;

        Ok(())
    }

    /// Change the current page to the first page.
    ///
    /// Will try lock `clips`.
    ///
    /// Will not trigger a tray update event.
    pub async fn first_page(&mut self) {
        self.current_page = 0;
    }

    /// Get all labels from the database
    ///   - return a vector of labels
    pub async fn get_all_labels(app: &AppHandle) -> Result<Vec<String>, anyhow::Error> {
        app.state::<DatabaseStateMutex>().get_all_labels().await
    }

    /// Get all the labels for a clip in the database
    ///   - return a vector of labels
    ///   - return None if the clip is not found
    pub async fn get_clip_labels(
        &self,
        app: &AppHandle,
        id: u64,
    ) -> Result<Option<Vec<String>>, anyhow::Error> {
        app.state::<DatabaseStateMutex>().get_clip_labels(id).await
    }

    /// Get a clip by id from database
    ///
    /// Get the latest clip if the id is none.
    /// None if the clip is not found.
    #[warn(unused_must_use)]
    pub async fn get_clip(
        &self,
        app: &AppHandle,
        id: Option<u64>,
    ) -> Result<Option<Clip>, anyhow::Error> {
        app.state::<DatabaseStateMutex>().get_clip(id).await
    }

    /// Get the clip data if current clip is not pinned clip,
    pub async fn get_current_clip(&self, app: &AppHandle) -> Result<Option<Clip>, anyhow::Error> {
        let current = self.current_clip;
        self.get_clip(app, current).await
    }

    /// Get the pos of the current clip.
    ///
    /// None if the current clip is a normal clip and is not in the list
    /// or the list is empty.
    pub async fn get_current_clip_pos(
        &self,
        app: &AppHandle,
    ) -> Result<Option<u64>, anyhow::Error> {
        let current = self.current_clip;
        self.get_id_pos_in_whole_list_of_ids(app, current).await
    }

    /// Get the position of the id in the database counting from the lower id to the higher id.
    /// The lowest id will return 0.
    /// The highest id will return the length of the list - 1.
    ///
    /// If the id is not in the list, return None.
    /// If the id is none, return the position of the latest clip.
    pub async fn get_id_pos_in_whole_list_of_ids(
        &self,
        app: &AppHandle,
        id: Option<u64>,
    ) -> Result<Option<u64>, anyhow::Error> {
        app.state::<DatabaseStateMutex>()
            .get_id_pos_in_whole_list_of_ids(id)
            .await
    }

    /// Get the maximum page number depend on
    /// the number of clips and the number of clips per page
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `database_connection`.
    pub async fn get_max_page(&self, app: &AppHandle) -> Result<u64, anyhow::Error> {
        // get the max page number
        // if there is no limit, return -1

        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clip_per_page = config.clip_per_page;
        drop(config);
        let len = Self::get_total_number_of_clip(app).await?;
        if len == 0 {
            return Ok(0);
        }
        let max_page = len / clip_per_page;
        if max_page * clip_per_page == len {
            return Ok(max_page - 1);
        }
        Ok(max_page)
    }

    /// Create a new clip in the database and return the id of the new clip
    pub async fn new_clip(
        &mut self,
        app: &AppHandle,
        clip: &mut Clip,
    ) -> Result<u64, anyhow::Error> {
        debug!("Create a new clip");

        let auto_delete_duplicate_clip = app.state::<ConfigMutex>();
        let auto_delete_duplicate_clip = auto_delete_duplicate_clip
            .config
            .lock()
            .await
            .auto_delete_duplicate_clip;
        let id = app
            .state::<DatabaseStateMutex>()
            .new_clip(clip, auto_delete_duplicate_clip)
            .await?;

        // change the current clip to the last one
        self.current_clip = Some(id);

        self.trigger_tray_update_event(app).await;
        Ok(id)
    }

    /// Jump to the next page in the tray
    ///
    /// Will not trigger a tray update event.
    pub async fn next_page(&mut self, app: &AppHandle) -> Result<(), anyhow::Error> {
        // switch to the next page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await?;
        self.switch_page(app, 1, max_page).await;

        self.trigger_tray_update_event(app).await;

        Ok(())
    }

    /// Jump to the previous page in the tray
    ///
    /// Will not trigger a tray update event.
    pub async fn prev_page(&mut self, app: &AppHandle) -> Result<(), anyhow::Error> {
        // switch to the previous page
        // if max_page is -1, it means there is no limit

        let max_page = self.get_max_page(app).await?;
        self.switch_page(app, -1, max_page).await;

        Ok(())
    }

    /// Change the clipboard to the selected clip,
    /// and also change the current clip to the selected clip
    ///
    /// If id is None, then change to the latest clip.
    #[warn(unused_must_use)]
    pub async fn select_clip(
        &mut self,
        app: &AppHandle,
        id: Option<u64>,
    ) -> Result<(), anyhow::Error> {
        // try get the clip
        let c = self.get_clip(app, id).await?;
        if c.is_none() {
            debug!("The requested ClipID: {:?} is not found", id);
            return Ok(());
        }
        let c = c.unwrap();

        self.current_clip = id;

        // change the clipboard
        copy_clip_to_clipboard_in(&c, app)?;

        Ok(())
    }

    /// Switch the tray to current_page + page,
    /// if the page is < 0, then switch to the first page,
    /// if the page is > max_page, then switch to the last page.
    pub async fn switch_page(&mut self, app: &AppHandle, page: i64, max_page: u64) {
        // switch to the page by the page number
        // if max_page is -1, it means there is no limit

        let target_page = self.current_page as i64 + page;
        if target_page < 0 {
            self.current_page = 0;
            return;
        }
        if target_page > max_page as i64 {
            self.current_page = max_page;
            return;
        }

        self.current_page = target_page as u64;

        self.trigger_tray_update_event(app).await;
    }

    /// Handle the clip board change event
    ///
    /// If the clipboard text is different from the most recent clip text,
    /// and is different from the current clip text,
    /// and is not empty,
    /// then create a new clip.
    /// Insert the new clip to the database.
    ///
    /// Will trigger a tray update event.
    #[warn(unused_must_use)]
    pub async fn update_clipboard(&mut self, app: &AppHandle) -> Result<(), anyhow::Error> {
        debug!("Clipboard changed");
        // get the current clip text
        let (clipboard_clip_type, clipboard_clip_text) = clip_data_from_system_clipboard(app)?;

        // if the current clip text is empty, then return
        if clipboard_clip_text.is_empty() {
            debug!("The clipboard text is empty, do not create a new clip");
            return Ok(());
        }

        let mut clipboard_clip =
            Clip::new_clip_with_id(0, &clipboard_clip_text, clipboard_clip_type)?;

        // get the current clip text
        let current_clip = self.get_current_clip(app).await?;
        // if the clip in the clipboard is the same as the current clip, then return
        if let Some(current_clip) = current_clip {
            if current_clip.clip_type == clipboard_clip.clip_type
                && current_clip.data == clipboard_clip.data
            {
                debug!(
                    "The clipboard text is the same as the current clip text, do not create a new clip"
                );
                return Ok(());
            }
        }

        // if the current clip text is the not same as the last clip text, then compare the last clip with the clip in the clipboard
        let latest_clip_id = Self::get_latest_clip_id(app).await?;
        if self.current_clip != latest_clip_id {
            let latest_clip = self.get_clip(app, latest_clip_id).await?;
            if let Some(latest_clip) = latest_clip {
                if latest_clip.clip_type == clipboard_clip.clip_type
                    && latest_clip.data == clipboard_clip.data
                {
                    debug!("The clipboard text is the same as the latest clip text, do not create a new clip");
                    return Ok(());
                }
            }
        }

        let id = self.new_clip(app, &mut clipboard_clip).await?;
        self.current_clip = Some(id);

        // update the tray
        let event_sender = app.state::<EventSender>();
        event_sender.send(CopyClipEvent::RebuildTrayMenuEvent).await;

        Ok(())
    }

    /// Get the id of a clip in the database by the position counting from the lower id to the higher id.
    /// 0 will return the lowest id.
    /// 1 will return the second lowest id.
    ///
    /// If the position is out of range, return None.
    pub async fn get_id_with_pos(app: &AppHandle, pos: u64) -> Result<Option<u64>, anyhow::Error> {
        app.state::<DatabaseStateMutex>().get_id_with_pos(pos).await
    }

    /// Update the normal clip tray
    ///
    /// Only used in `self.update_tray()`
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    async fn update_normal_clip_tray(
        &mut self,
        app: &AppHandle,
        clips_per_page: u64,
        whole_list_of_ids_len: u64,
        max_clip_length: u64,
        current_page: u64,
    ) -> Result<(), anyhow::Error> {
        // get the current page clips
        let mut current_page_clips: Vec<Clip> = Vec::new();
        for i in 0..clips_per_page {
            let pos = whole_list_of_ids_len as i64 - (current_page * clips_per_page + i + 1) as i64;
            if pos < 0 {
                break;
            }
            let pos = pos as u64;
            let clip_id = match Self::get_id_with_pos(app, pos).await? {
                Some(clip_id) => clip_id,
                None => {
                    break;
                }
            };

            let clip = self.get_clip(app, Some(clip_id)).await?;
            if clip.is_none() {
                break;
            }
            current_page_clips.push(clip.unwrap());
        }

        // put text in
        for i in 0..current_page_clips.len() {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);

            let c = current_page_clips.get(i);
            if c.is_none() {
                continue;
            }
            let c = c.unwrap();

            let text = clip::trimming_clip_text(&c.search_text, max_clip_length);
            tray_clip_sub_menu.set_title(text)?;
        }

        // clean out the rest of the tray
        for i in current_page_clips.len()..clips_per_page as usize {
            let tray_id = "tray_clip_".to_string() + &i.to_string();
            let tray_clip_sub_menu = app.tray_handle().get_item(&tray_id);
            tray_clip_sub_menu.set_title("".to_string())?
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
        whole_list_of_ids_len: u64,
        current_page: u64,
        whole_pages: u64,
    ) -> Result<(), Error> {
        let tray_page_info_item = app.tray_handle().get_item("page_info");
        let tray_page_info_title = format!(
            "{}: {}, {}: {}/{}",
            t!("tray_menu.total_clips"),
            whole_list_of_ids_len,
            t!("tray_menu.current_page"),
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
    async fn update_pinned_clips(
        &self,
        app: &AppHandle,
        max_clip_length: u64,
    ) -> Result<(), anyhow::Error> {
        // update pinned clips in the tray
        let pinned_clips_len = Self::get_label_clip_number(app, "pinned").await?;
        debug!("Pinned clips length: {}", pinned_clips_len);
        for i in 0..pinned_clips_len {
            debug!("Pinned clip: {}", i);
            let pinned_clip = match Self::get_label_clip_id_with_pos(app, "pinned", i).await? {
                Some(pinned_clip) => pinned_clip,
                None => {
                    break;
                }
            };

            let pinned_clip = match self.get_clip(app, Some(pinned_clip)).await? {
                Some(pinned_clip) => pinned_clip,
                None => {
                    break;
                }
            };

            let pinned_clip = clip::trimming_clip_text(&pinned_clip.search_text, max_clip_length);
            let pinned_clip_item: tauri::SystemTrayMenuItemHandle =
                app.tray_handle().get_item(&format!("pinned_clip_{}", i));
            pinned_clip_item.set_title(pinned_clip)?;
        }

        Ok(())
    }

    /// Update the favourite clips in the tray
    ///
    /// Only used in `self.update_tray()`
    async fn update_favourite_clips(
        &self,
        app: &AppHandle,
        max_clip_length: u64,
    ) -> Result<(), anyhow::Error> {
        // update favourite clips in the tray
        let favourite_clips_len = Self::get_label_clip_number(app, "favourite").await?;
        for i in 0..favourite_clips_len {
            let favourite_clip = match Self::get_label_clip_id_with_pos(app, "favourite", i).await?
            {
                Some(favourite_clip) => favourite_clip,
                None => {
                    break;
                }
            };

            let favourite_clip = match self.get_clip(app, Some(favourite_clip)).await? {
                Some(favourite_clip) => favourite_clip,
                None => {
                    break;
                }
            };

            let favourite_clip_text =
                clip::trimming_clip_text(&favourite_clip.search_text, max_clip_length);
            let favourite_clip_id = "favourite_clip_".to_string() + &i.to_string();
            let favourite_clip_item: tauri::SystemTrayMenuItemHandle =
                app.tray_handle().get_item(&favourite_clip_id);
            favourite_clip_item.set_title(favourite_clip_text)?
        }

        Ok(())
    }

    /// Update the tray with all the current clips, the pinned clips, the favourite clips,
    /// and other data
    ///
    /// Will try lock `app.state::<ConfigMutex>()` and `clips` and `database_connection`
    #[warn(unused_must_use)]
    pub async fn update_tray(&mut self, app: &AppHandle) -> Result<(), anyhow::Error> {
        debug!("Starting to update the tray");
        // get the clips per page configuration
        let config = app.state::<ConfigMutex>();
        let config = config.config.lock().await;
        let clips_per_page = config.clip_per_page;
        let max_clip_length = config.clip_max_show_length;
        drop(config);

        // get the current page
        debug!("Getting the current page number");
        let mut current_page = self.current_page;
        let whole_pages = self.get_max_page(app).await?;
        let whole_list_of_ids_len = Self::get_total_number_of_clip(app).await?;
        // if the current page bigger than the whole pages, then calculate the current page, regarding to current_clip_pos
        if current_page > whole_pages {
            // get the current clip pos
            let current_clip_pos = self.get_current_clip_pos(app).await?;
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
                current_page = (whole_list_of_ids_len - current_clip_pos - 1) / clips_per_page;
            }
            self.current_page = current_page;
        }

        debug!("Updating the tray page info");
        self.update_tray_page_info(app, whole_list_of_ids_len, current_page, whole_pages)
            .await?;
        debug!("Updating the pinned clips");
        self.update_pinned_clips(app, max_clip_length).await?;
        debug!("Updating the normal clips");
        self.update_normal_clip_tray(
            app,
            clips_per_page,
            whole_list_of_ids_len,
            max_clip_length,
            current_page,
        )
        .await?;
        debug!("Updating the favourite clips");
        self.update_favourite_clips(app, max_clip_length).await?;

        debug!("Finish updating the tray");

        Ok(())
    }
}

/// The current clip text in the system clipboard
///
/// The first element is the clip type, the second element is
/// - the text if the clip type is text
/// - the rtf if the clip type is rtf
/// - the html if the clip type is html
/// - the json formatted vec<FileURI> if the clip type is file
/// - a string of the image that indicate where the image is saved if the clip type is image
fn clip_data_from_system_clipboard(app: &AppHandle) -> Result<(ClipType, String), Error> {
    let clipboard_manager = app.state::<tauri_plugin_clipboard::ClipboardManager>();
    if match clipboard_manager.has_image() {
        Ok(has_image) => has_image,
        Err(err) => {
            return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
        }
    } {
        match clipboard_manager.read_image_binary() {
            Ok(clip) => {
                // get path of the image
                let user_data_dir = match app.path_resolver().app_data_dir() {
                    Some(user_data_dir) => user_data_dir,
                    None => return Err(Error::GetAppDataDirErr),
                };
                let image_path = image_clip::store_img_return_path(user_data_dir, &clip)?;
                return Ok((ClipType::Image, image_path));
            }
            Err(err) => return Err(Error::ReadFromSystemClipboardErr(err.to_string())),
        }
    }

    if match clipboard_manager.has_files() {
        Ok(has_files) => has_files,
        Err(err) => {
            return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
        }
    } {
        match clipboard_manager.read_files_uris() {
            Ok(clip) => {
                // convert the clip to json
                let clip = serde_json::to_string(&clip).unwrap();
                return Ok((ClipType::File, clip));
            }
            Err(err) => return Err(Error::ReadFromSystemClipboardErr(err.to_string())),
        }
    }

    if match clipboard_manager.has_html() {
        Ok(has_html) => has_html,
        Err(err) => {
            return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
        }
    } {
        match clipboard_manager.read_html() {
            Ok(clip) => return Ok((ClipType::Html, clip)),
            Err(err) => return Err(Error::ReadFromSystemClipboardErr(err.to_string())),
        }
    }

    if match clipboard_manager.has_rtf() {
        Ok(has_rtf) => has_rtf,
        Err(err) => {
            return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
        }
    } {
        match clipboard_manager.read_rtf() {
            Ok(clip) => return Ok((ClipType::Rtf, clip)),
            Err(err) => return Err(Error::ReadFromSystemClipboardErr(err.to_string())),
        }
    }

    if match clipboard_manager.has_text() {
        Ok(has_text) => has_text,
        Err(err) => {
            return Err(Error::ReadFromSystemClipboardErr(err.to_string()));
        }
    } {
        match clipboard_manager.read_text() {
            Ok(clip) => return Ok((ClipType::Text, clip)),
            Err(err) => return Err(Error::ReadFromSystemClipboardErr(err.to_string())),
        }
    }

    // The only possible type left is unknown
    Ok((ClipType::Text, "".to_string()))
}
