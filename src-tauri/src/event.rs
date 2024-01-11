use log::{debug, error};
use tauri::async_runtime::{Receiver, Sender};

use tauri::{api::notification::Notification, AppHandle, Manager};

use crate::{
    clip::clip_data::ClipData,
    config::ConfigMutex,
    log::{panic_app, LogLevel},
    systray::{create_tray_menu, handle_menu_item_click, send_tray_update_event},
};

/// all the events that can be sent to the event daemon
#[derive(Debug)]
pub enum CopyClipEvent {
    /// update the clips in the tray menu
    TrayUpdateEvent,
    /// rebuild the tray menu
    RebuildTrayMenuEvent,
    /// save config event
    SaveConfigEvent,
    /// clipboard change event
    ClipboardChangeEvent,
    /// tray menu item click event,
    /// the data is the id the tray item
    TrayMenuItemClickEvent(String),
    /// log
    LogEvent(LogLevel, String),
    /// send notification event
    /// the data is the notification message
    SendNotificationEvent(String),
    /// pinned clips changed event
    /// no data
    /// this event is sent when the pinned clips changed
    /// should update the pinned clips in the tray menu
    PinnedClipsChangedEvent,
}

/// the event sender
/// the sender is wrapped in a mutex to allow it to be used in multiple threads
/// the mutex is locked when sending an event
/// the sender should in pair with a receiver that is used in the event daemon
#[derive(Clone)]
pub struct EventSender {
    pub tx: Sender<CopyClipEvent>,
}

impl EventSender {
    pub fn new(tx: Sender<CopyClipEvent>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, event: CopyClipEvent) {
        let res = self.tx.send(event).await;
        if let Err(err) = res {
            error!("Failed to send event, error: {}", err);
            panic_app(&format!("Failed to send event, error: {}", err));
        }
    }
}

/// Send an event to the event daemon
///
/// Will panic if the event cannot be sent
pub fn event_sender(app: &AppHandle, event: CopyClipEvent) {
    let event_sender = app.state::<EventSender>();
    let tx = event_sender.tx.clone();
    tauri::async_runtime::spawn(async move {
        let res = tx.send(event).await;
        if let Err(err) = res {
            error!("Failed to send event, error: {}", err);
            panic_app(&format!("Failed to send event, error: {}", err));
        }
    });
}

/// the event daemon
/// the daemon is a loop that waits for events to be sent to it
pub async fn event_daemon(mut rx: Receiver<CopyClipEvent>, app: &AppHandle) {
    loop {
        let event = rx.recv().await;
        if event.is_none() {
            continue;
        }
        let event = event.unwrap();
        debug!("Get event: {:?}", event);
        let app = app.app_handle();
        match event {
            // update the clips in the tray menu
            CopyClipEvent::TrayUpdateEvent => tauri::async_runtime::spawn(async move {
                let clip_data = app.state::<ClipData>();
                let res = clip_data.update_tray(&app).await;
                if let Err(err) = res {
                    panic_app(&format!(
                        "Failed to update tray menu, error: {}",
                        err.message()
                    ));
                }
            }),
            // rebuild the tray menu
            CopyClipEvent::RebuildTrayMenuEvent => tauri::async_runtime::spawn(async move {
                // get number of clips to show from config
                let page_len = app.state::<ConfigMutex>();
                let page_len = page_len.config.lock().await.clip_per_page;
                // get the number of pinned clips
                let pinned_clips = app.state::<ClipData>();
                let pinned_clips = pinned_clips.clips.lock().await.pinned_clips.len();
                // get the number of favourite clips
                let favourite_clips = app.state::<ClipData>();
                let favourite_clips = favourite_clips.clips.lock().await.favourite_clips.len();
                // get paused state
                let paused = app.state::<ConfigMutex>();
                let paused = paused.config.lock().await.pause_monitoring;

                let res = app.tray_handle().set_menu(create_tray_menu(
                    page_len,
                    pinned_clips as i64,
                    favourite_clips as i64,
                    paused,
                ));
                if res.is_err() {
                    panic_app(&format!(
                        "Failed to set tray menu, error: {}",
                        res.err().unwrap()
                    ));
                }
                // initial the tray
                send_tray_update_event(&app);
            }),
            // pinned clips changed event
            CopyClipEvent::PinnedClipsChangedEvent => tauri::async_runtime::spawn(async move {
                // send rebuild tray menu event
                let event = app.state::<EventSender>();
                event.send(CopyClipEvent::RebuildTrayMenuEvent).await;
            }),
            // save config event
            CopyClipEvent::SaveConfigEvent => tauri::async_runtime::spawn(async move {
                let config = app.state::<ConfigMutex>();
                let config = config.config.lock().await;
                let res = config.save_config(&app);
                drop(config);
                if res.is_err() {
                    panic_app(&format!("Failed to {}", res.err().unwrap().message()));
                }
            }),
            // log
            CopyClipEvent::LogEvent(level, msg) => tauri::async_runtime::spawn(async move {
                log::log!(log::Level::from(level), "{msg}");
            }),
            // clipboard change event
            CopyClipEvent::ClipboardChangeEvent => tauri::async_runtime::spawn(async move {
                debug!("Clipboard change event");
                let config = app.state::<ConfigMutex>();
                let config = config.config.lock().await;
                if !config.pause_monitoring {
                    drop(config);
                    let clip_data = app.state::<ClipData>();
                    let res = clip_data.update_clipboard(&app).await;
                    if let Err(err) = res {
                        panic_app(&format!(
                            "Failed to update clipboard, error: {}",
                            err.message()
                        ));
                    }
                }
            }),
            // tray menu item click event
            CopyClipEvent::TrayMenuItemClickEvent(id) => tauri::async_runtime::spawn(async move {
                handle_menu_item_click(&app, id).await;
            }),
            CopyClipEvent::SendNotificationEvent(msg) => tauri::async_runtime::spawn(async move {
                #[cfg(debug_assertions)]
                log::debug!("Notification: {}", msg);

                let res = Notification::new(&app.config().tauri.bundle.identifier)
                    .title(msg)
                    .icon("icons/clip.png")
                    .notify(&app);
                if let Err(err) = res {
                    #[cfg(debug_assertions)]
                    println!("Error: {}", err);

                    log::error!("Error: {}", err);
                }
            }),
        };
    }
}
