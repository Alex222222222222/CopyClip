use anyhow::Ok;
use once_cell::sync::OnceCell;
use tauri::{
    async_runtime::Mutex,
    menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder},
    tray::TrayIcon,
    AppHandle, Manager,
};

use crate::clip_frontend::clip_data::ClipState;

/// The struct to manage the system tray menu.
/// The first init will set the tray icon.
/// The other fields, if it is once cell, then it should be only initialized once.
/// If it is not once cell, then it may be changed.
pub struct SystemTrayMenuMutex {
    pub menu: Mutex<SystemTrayMenu>,
}

pub struct SystemTrayMenu {
    tray: OnceCell<TrayIcon>,
    notice_select: OnceCell<MenuItem<tauri::Wry>>,
    /// a list of pinned clips
    pinned_clips: Vec<MenuItem<tauri::Wry>>,
    /// a submenu of favourite clips
    favourite_clips: Vec<MenuItem<tauri::Wry>>,
    /// a list of recent clips
    recent_clips: Vec<MenuItem<tauri::Wry>>,
    page_info: Option<MenuItem<tauri::Wry>>,
    prev_page: OnceCell<MenuItem<tauri::Wry>>,
    next_page: OnceCell<MenuItem<tauri::Wry>>,
    first_page: OnceCell<MenuItem<tauri::Wry>>,
    preferences: OnceCell<MenuItem<tauri::Wry>>,
    search: OnceCell<MenuItem<tauri::Wry>>,
    pause: Option<MenuItem<tauri::Wry>>,
    quit: OnceCell<MenuItem<tauri::Wry>>,
}

impl Default for SystemTrayMenuMutex {
    fn default() -> Self {
        Self {
            menu: Mutex::new(SystemTrayMenu::default()),
        }
    }
}

impl Default for SystemTrayMenu {
    fn default() -> Self {
        Self {
            tray: OnceCell::new(),
            notice_select: OnceCell::new(),
            pinned_clips: Vec::new(),
            favourite_clips: Vec::new(),
            recent_clips: Vec::new(),
            page_info: None,
            prev_page: OnceCell::new(),
            next_page: OnceCell::new(),
            first_page: OnceCell::new(),
            preferences: OnceCell::new(),
            search: OnceCell::new(),
            pause: None,
            quit: OnceCell::new(),
        }
    }
}

impl SystemTrayMenuMutex {
    /// Set the tray icon.
    pub async fn set_tray_icon(&self, tray: TrayIcon) -> anyhow::Result<()> {
        let menu = self.menu.lock().await;
        menu.tray
            .set(tray)
            .map_err(|_| anyhow::anyhow!("tray icon already set"))?;

        Ok(())
    }

    /// update the current state of the system tray menu.
    /// the will only change the entries in the struct, not the actual tray icon.
    /// to update the tray icon, call `update_tray_icon`, which will automatically invoke `update_tray_state`.
    async fn update_tray_state(&self, app: &AppHandle) -> anyhow::Result<()> {
        // update the pinned clips
        self.update_pinned_clips(app).await?;
        // update the favourite clips
        self.update_favourite_clips(app).await?;
        // update the recent clips
        self.update_recent_clips(app).await?;

        self.update_page_info(app).await?;
        self.update_pause_info(app).await?;

        Ok(())
    }

    /// update the recent clips.
    /// This function should only be called by `update_tray_state`.
    ///
    /// Parameters:
    /// - `app`: the app handle
    async fn update_recent_clips(&self, app: &AppHandle) -> anyhow::Result<()> {
        // get the necessary info
        let page_len = app
            .state::<crate::config::ConfigMutex>()
            .config
            .lock()
            .await
            .clip_per_page;
        let whole_list_of_ids_len = ClipState::get_total_number_of_clip(app).await?;
        let current_page = app
            .state::<crate::clip_frontend::clip_data::ClipStateMutex>()
            .clip_state
            .lock()
            .await
            .current_page;
        let max_clip_length = app
            .state::<crate::config::ConfigMutex>()
            .config
            .lock()
            .await
            .clip_max_show_length;

        let mut tray_menu = self.menu.lock().await;
        tray_menu.recent_clips.clear();

        // add the clips
        for i in 0..page_len {
            let pos = whole_list_of_ids_len as i64 - (current_page * page_len + i + 1) as i64;
            if pos < 0 {
                break;
            }
            let pos = pos as u64;
            let clip_id = match ClipState::get_id_with_pos(app, pos).await? {
                Some(clip_id) => clip_id,
                None => {
                    break;
                }
            };

            let clip = app.state::<crate::clip_frontend::clip_data::ClipStateMutex>();
            let clip = clip.clip_state.lock().await;
            let clip = clip.get_clip(app, Some(clip_id)).await?;
            if clip.is_none() {
                break;
            }
            let clip = clip.unwrap();

            let clip = clip::trimming_clip_text(&clip.search_text, max_clip_length);
            let recent_clip_item =
                MenuItemBuilder::with_id(format!("clip_{}", clip_id), clip).build(app)?;
            tray_menu.recent_clips.push(recent_clip_item);
        }

        Ok(())
    }

    /// update the favourite clips.
    /// This function should only be called by `update_tray_state`.
    ///
    /// Parameters:
    /// - `app`: the app handle
    async fn update_favourite_clips(&self, app: &AppHandle) -> anyhow::Result<()> {
        // get the necessary info
        let favourite_clips_num: u64 = ClipState::get_label_clip_number(app, "favourite").await?;
        let max_clip_length = app
            .state::<crate::config::ConfigMutex>()
            .config
            .lock()
            .await
            .clip_max_show_length;

        // insert the favourite clips into the menu
        let mut tray_menu = self.menu.lock().await;
        tray_menu.favourite_clips.clear();

        for i in 0..favourite_clips_num {
            let clip_id = match ClipState::get_label_clip_id_with_pos(app, "favourite", i).await? {
                Some(clip_id) => clip_id,
                None => {
                    break;
                }
            };

            let clip_data = app.state::<crate::clip_frontend::clip_data::ClipStateMutex>();
            let clip_data = clip_data.clip_state.lock().await;
            let clip = match clip_data.get_clip(app, Some(clip_id)).await? {
                Some(clip) => clip,
                None => {
                    break;
                }
            };
            drop(clip_data);

            let clip = clip::trimming_clip_text(&clip.search_text, max_clip_length);
            let favourite_clip_item =
                MenuItemBuilder::with_id(format!("clip_{}", clip_id), clip).build(app)?;
            tray_menu.favourite_clips.push(favourite_clip_item);
        }

        Ok(())
    }

    /// update the pinned clips.
    /// This function should only be called by `update_tray_state`.
    ///
    /// Parameters:
    /// - `app`: the app handle
    async fn update_pinned_clips(&self, app: &AppHandle) -> anyhow::Result<()> {
        // get the necessary info
        let pinned_clips_num: u64 = ClipState::get_label_clip_number(app, "pinned").await?;
        let max_clip_length = app.state::<crate::config::ConfigMutex>();
        let max_clip_length = max_clip_length.config.lock().await;
        let max_clip_length = max_clip_length.clip_max_show_length;

        // insert the pinned clips into the menu
        let mut tray_menu = self.menu.lock().await;
        tray_menu.pinned_clips.clear();

        for i in 0..pinned_clips_num {
            let clip_id = match ClipState::get_label_clip_id_with_pos(app, "pinned", i).await? {
                Some(clip_id) => clip_id,
                None => {
                    break;
                }
            };

            let clip_data = app.state::<crate::clip_frontend::clip_data::ClipStateMutex>();
            let clip_data = clip_data.clip_state.lock().await;
            let clip = match clip_data.get_clip(app, Some(clip_id)).await? {
                Some(clip) => clip,
                None => {
                    break;
                }
            };
            drop(clip_data);

            let clip = clip::trimming_clip_text(&clip.search_text, max_clip_length);
            let pinned_clip_item =
                MenuItemBuilder::with_id(format!("clip_{}", clip_id), clip).build(app)?;
            tray_menu.pinned_clips.push(pinned_clip_item);
        }

        Ok(())
    }

    /// update the pause info, which is the current pause state.
    /// This function should only be called by `update_tray_state`.
    ///
    /// Parameters:
    /// - `app`: the app handle
    async fn update_pause_info(&self, app: &AppHandle) -> anyhow::Result<()> {
        // get the necessary info
        let paused = app.state::<crate::config::ConfigMutex>();
        let paused = paused.config.lock().await.pause_monitoring;

        // build the pause info
        let pause_title = if paused {
            t!("tray_menu.resume_monitoring")
        } else {
            t!("tray_menu.pause_monitoring")
        };
        let pause = MenuItemBuilder::with_id("pause".to_string(), pause_title).build(app)?;

        // set the pause info
        let mut system_tray_menu = self.menu.lock().await;
        system_tray_menu.pause = Some(pause);

        Ok(())
    }

    /// update the page info, which is the current page number and the total number of pages.
    /// This function should only be called by `update_tray_state`.
    /// The format of the page info is:
    ///
    /// `Total clips: 0, Current page: 0/0`
    ///
    /// Parameters:
    /// - `app`: the app handle
    async fn update_page_info(&self, app: &AppHandle) -> anyhow::Result<()> {
        // get the necessary info
        let whole_list_of_ids_len = ClipState::get_total_number_of_clip(app).await?;
        let clip_data = app.state::<crate::clip_frontend::clip_data::ClipStateMutex>();
        let clip_data = clip_data.clip_state.lock().await;
        let current_page = clip_data.current_page;
        let whole_pages = clip_data.get_max_page(app).await?;
        drop(clip_data);

        // build the page info
        let tray_page_info_title = format!(
            "{}: {}, {}: {}/{}",
            t!("tray_menu.total_clips"),
            whole_list_of_ids_len,
            t!("tray_menu.current_page"),
            current_page + 1,
            whole_pages + 1
        );
        let page_info = MenuItemBuilder::with_id("page_info".to_string(), tray_page_info_title)
            .enabled(false)
            .build(app)?;

        // set the page info
        let mut system_tray_menu = self.menu.lock().await;
        system_tray_menu.page_info = Some(page_info);

        Ok(())
    }

    /// Init the item that only need to be initialized once.
    pub async fn init_once_cell(&self, app: &AppHandle) -> anyhow::Result<()> {
        let system_tray_menu = self.menu.lock().await;

        let notice_select =
            MenuItemBuilder::with_id("notice_select".to_string(), t!("tray_menu.notice_select"))
                .enabled(false)
                .build(app)?;
        system_tray_menu
            .notice_select
            .set(notice_select)
            .map_err(|_| anyhow::anyhow!("notice_select already set"))?;

        let prev_page =
            MenuItemBuilder::with_id("prev_page".to_string(), t!("tray_menu.prev_page"))
                .accelerator("CommandOrControl+A")
                .build(app)?;
        system_tray_menu
            .prev_page
            .set(prev_page)
            .map_err(|_| anyhow::anyhow!("prev_page already set"))?;

        let next_page =
            MenuItemBuilder::with_id("next_page".to_string(), t!("tray_menu.next_page"))
                .accelerator("CommandOrControl+D")
                .build(app)?;
        system_tray_menu
            .next_page
            .set(next_page)
            .map_err(|_| anyhow::anyhow!("next_page already set"))?;

        let first_page =
            MenuItemBuilder::with_id("first_page".to_string(), t!("tray_menu.first_page"))
                .build(app)?;
        system_tray_menu
            .first_page
            .set(first_page)
            .map_err(|_| anyhow::anyhow!("first_page already set"))?;

        let preferences =
            MenuItemBuilder::with_id("preferences".to_string(), t!("tray_menu.preferences"))
                .build(app)?;
        system_tray_menu
            .preferences
            .set(preferences)
            .map_err(|_| anyhow::anyhow!("preferences already set"))?;
        let search =
            MenuItemBuilder::with_id("search".to_string(), t!("tray_menu.search")).build(app)?;
        system_tray_menu
            .search
            .set(search)
            .map_err(|_| anyhow::anyhow!("search already set"))?;

        let quit = MenuItemBuilder::with_id("quit".to_string(), t!("tray_menu.quit")).build(app)?;
        system_tray_menu
            .quit
            .set(quit)
            .map_err(|_| anyhow::anyhow!("quit already set"))?;

        Ok(())
    }

    /// update the tray icon.
    pub async fn update_tray_icon(&self, app: &AppHandle) -> anyhow::Result<()> {
        self.update_tray_state(app).await?;

        let menu = self.build_tray_menu(app).await?;
        let system_tray_menu = self.menu.lock().await;

        let tray = system_tray_menu
            .tray
            .get()
            .ok_or_else(|| anyhow::anyhow!("tray icon not set"))?;
        tray.set_menu(Some(menu))
            .map_err(|_| anyhow::anyhow!("failed to set menu"))?;

        Ok(())
    }

    /// build a tray menu from the current state
    async fn build_tray_menu(&self, app: &AppHandle) -> anyhow::Result<Menu<tauri::Wry>> {
        let system_tray_menu = self.menu.lock().await;

        let mut menu = MenuBuilder::new(app);

        let notice_select = system_tray_menu.notice_select.get();
        if let Some(notice_select) = notice_select {
            menu = menu.item(notice_select).separator();
        }

        if !system_tray_menu.pinned_clips.is_empty() {
            for clip in &system_tray_menu.pinned_clips {
                menu = menu.item(clip);
            }
            menu = menu.separator();
        }

        if !system_tray_menu.favourite_clips.is_empty() {
            let mut favourite = tauri::menu::SubmenuBuilder::new(app, t!("tray_menu.favourite"));
            for clip in &system_tray_menu.favourite_clips {
                favourite = favourite.item(clip);
            }
            let favourite = favourite.build()?;
            menu = menu.item(&favourite).separator();
        }

        if !system_tray_menu.recent_clips.is_empty() {
            for clip in &system_tray_menu.recent_clips {
                menu = menu.item(clip);
            }
            menu = menu.separator();
        }

        if let Some(page_info) = &system_tray_menu.page_info {
            menu = menu.item(page_info).separator();
        }

        if let Some(prev_page) = system_tray_menu.prev_page.get() {
            menu = menu.item(prev_page);
        }

        if let Some(next_page) = system_tray_menu.next_page.get() {
            menu = menu.item(next_page);
        }

        if let Some(first_page) = system_tray_menu.first_page.get() {
            menu = menu.item(first_page).separator();
        }

        menu = menu.separator();

        if let Some(preferences) = system_tray_menu.preferences.get() {
            menu = menu.item(preferences);
        }

        if let Some(search) = system_tray_menu.search.get() {
            menu = menu.item(search);
        }

        if let Some(pause) = &system_tray_menu.pause {
            menu = menu.item(pause);
        }

        menu = menu.separator();

        if let Some(quit) = system_tray_menu.quit.get() {
            menu = menu.item(quit);
        }

        let menu = menu.build()?;

        Ok(menu)
    }
}
