use std::sync::{mpsc::Sender, Mutex};

use tauri::{api::notification::Notification, AppHandle, Manager};

use crate::{
    clip::ClipDataMutex,
    config::ConfigMutex,
    log::{panic_app, LogLevel},
    systray::{create_tray_menu, handle_menu_item_click, send_tray_update_event},
};

/// all the events that can be sent to the event daemon
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
}

/// the event sender
/// the sender is wrapped in a mutex to allow it to be used in multiple threads
/// the mutex is locked when sending an event
/// the sender should in pair with a receiver that is used in the event daemon
pub struct EventSender {
    tx: Mutex<Sender<CopyClipEvent>>,
}

impl EventSender {
    pub fn new(tx: Sender<CopyClipEvent>) -> Self {
        Self { tx: Mutex::new(tx) }
    }

    pub fn send(&self, event: CopyClipEvent) {
        let tx = self.tx.lock().unwrap();
        tx.send(event).unwrap();
        drop(tx);
    }
}

/// the event daemon
/// the daemon is a loop that waits for events to be sent to it
pub async fn event_daemon(rx: std::sync::mpsc::Receiver<CopyClipEvent>, app: &AppHandle) {
    loop {
        let event = rx.recv().unwrap();
        match event {
            // update the clips in the tray menu
            CopyClipEvent::TrayUpdateEvent => {
                let clip_data = app.state::<ClipDataMutex>();
                let mut clip_data = clip_data.clip_data.lock().await;
                let res = clip_data.update_tray(app).await;
                drop(clip_data);
                if let Err(err) = res {
                    panic_app(&format!(
                        "Failed to update tray menu, error: {}",
                        err.message()
                    ));
                }
            }
            // rebuild the tray menu
            CopyClipEvent::RebuildTrayMenuEvent => {
                // get number of clips to show from config
                let num = app.state::<ConfigMutex>();
                let num = num.config.lock().await.clip_per_page;
                let res = app.tray_handle().set_menu(create_tray_menu(num));
                if res.is_err() {
                    panic_app(&format!(
                        "Failed to set tray menu, error: {}",
                        res.err().unwrap()
                    ));
                }
                // initial the tray
                send_tray_update_event(app);
            }
            // save config event
            CopyClipEvent::SaveConfigEvent => {
                let config = app.state::<ConfigMutex>();
                let config = config.config.lock().await;
                let res = config.save_config(app);
                drop(config);
                if res.is_err() {
                    panic_app(&format!("Failed to {}", res.err().unwrap().message()));
                }
            }
            // log
            CopyClipEvent::LogEvent(level, msg) => {
                log::log!(log::Level::from(level), "{msg}");
            }
            // clipboard change event
            CopyClipEvent::ClipboardChangeEvent => {
                let clip_data = app.state::<ClipDataMutex>();
                let mut clip_data = clip_data.clip_data.lock().await;
                let res = clip_data.update_clipboard(app).await;
                drop(clip_data);
                if let Err(err) = res {
                    panic_app(&format!(
                        "Failed to update clipboard, error: {}",
                        err.message()
                    ));
                }
            }
            // tray menu item click event
            CopyClipEvent::TrayMenuItemClickEvent(id) => {
                handle_menu_item_click(app, id).await;
            }
            CopyClipEvent::SendNotificationEvent(msg) => {
                #[cfg(debug_assertions)]
                log::debug!("Notification: {}", msg);

                let res = Notification::new(&app.config().tauri.bundle.identifier)
                    .title(msg)
                    .icon("icons/clip.png")
                    .show();
                if let Err(err) = res {
                    #[cfg(debug_assertions)]
                    println!("Error: {}", err);

                    log::error!("Error: {}", err);
                }
            }
        }
    }
}
