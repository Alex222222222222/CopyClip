use log::{debug, error};
use tauri::async_runtime::{Receiver, Sender};

use tauri::{AppHandle, Manager};
use tauri_plugin_logging::panic_app;
use tauri_plugin_notification::NotificationExt;

use crate::clip_frontend::clip_data::{ClipState, ClipStateMutex};
use crate::{
    config::ConfigMutex,
    systray::{create_tray_menu, handle_menu_item_click},
};

/// all the events that can be sent to the event daemon
#[derive(Debug)]
pub enum CopyClipEvent {
    /// rebuild the tray menu
    RebuildTrayMenuEvent,
    /// save config event
    SaveConfigEvent,
    /// clipboard change event
    ClipboardChangeEvent,
    /// tray menu item click event,
    /// the data is the id the tray item
    TrayMenuItemClickEvent(String),
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
pub async fn event_daemon<R: tauri::Runtime>(mut rx: Receiver<CopyClipEvent>, app: &AppHandle) {
    loop {
        let event = rx.recv().await;
        if event.is_none() {
            continue;
        }
        let event = event.unwrap();
        debug!("Get event: {:?}", event);
        let app = app.app_handle().clone();
        match event {
            // rebuild the tray menu
            CopyClipEvent::RebuildTrayMenuEvent => tauri::async_runtime::spawn(async move {
                // get number of clips to show from config
                let page_len = app.state::<ConfigMutex>();
                let page_len = page_len.config.lock().await.clip_per_page;
                // get the number of pinned clips
                let pinned_clips = match ClipState::get_label_clip_number(&app, "pinned").await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Failed to get pinned clips, error: {}", err);
                        return;
                    }
                };
                // get the number of favourite clips
                let favourite_clips =
                    match ClipState::get_label_clip_number(&app, "favourite").await {
                        Ok(res) => res,
                        Err(err) => {
                            error!("Failed to get favourite clips, error: {}", err);
                            return;
                        }
                    };
                // get paused state
                let paused = app.state::<ConfigMutex>();
                let paused = paused.config.lock().await.pause_monitoring;

                // TODO handle error
                let menu = create_tray_menu(
                    page_len as i64,
                    pinned_clips as i64,
                    favourite_clips as i64,
                    paused,
                    &app,
                ).unwrap();
                // TODO: handle error and check if the menu is created
                let res = tauri::tray::TrayIconBuilder::new()
                    .menu(&menu)
                    .menu_on_left_click(true)
                    .build(&app);
                if res.is_err() {
                    panic_app(&format!(
                        "Failed to set tray menu, error: {}",
                        res.err().unwrap()
                    ));
                }

                let clip_data = app.state::<ClipStateMutex>();
                let mut clip_data = clip_data.clip_state.lock().await;

                let res = clip_data.update_tray(&app).await;
                if let Err(err) = res {
                    panic_app(&format!("Failed to update tray menu, error: {}", err));
                }
                drop(clip_data);
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
            // clipboard change event
            CopyClipEvent::ClipboardChangeEvent => tauri::async_runtime::spawn(async move {
                debug!("Clipboard change event");
                let config = app.state::<ConfigMutex>();
                let config = config.config.lock().await;
                let paused = config.pause_monitoring;
                debug!("Paused: {}", paused);
                drop(config);
                if !paused {
                    debug!("Clipboard change event, not paused");
                    let clip_data = app.state::<ClipStateMutex>();
                    let mut clip_data = clip_data.clip_state.lock().await;

                    let res = clip_data.update_clipboard(&app).await;
                    debug!("Clipboard updated");
                    if let Err(err) = res {
                        panic_app(&format!("Failed to update clipboard, error: {}", err,));
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

                let res = app
                    .notification()
                    .builder()
                    .title("CopyClip")
                    .body(msg)
                    .icon("icons/clip.png")
                    .show();
                if let Err(err) = res {
                    #[cfg(debug_assertions)]
                    println!("Error: {}", err);

                    log::error!("Error: {}", err);
                }
            }),
        };
    }
}
