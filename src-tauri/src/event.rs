use std::sync::{mpsc::Sender, Mutex};

use tauri::{AppHandle, Manager};

use crate::{
    clip::ClipDataMutex,
    config::ConfigMutex,
    systray::{create_tray_menu, send_tray_update_event},
};

/// all the events that can be sent to the event daemon
pub enum CopyClipEvent {
    /// update the clips in the tray menu
    TrayUpdateEvent,
    /// rebuild the tray menu
    RebuildTrayMenuEvent,
    // TODO add save config event
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
    }
}

/// the event daemon
/// the daemon is a loop that waits for events to be sent to it
pub fn event_daemon(rx: std::sync::mpsc::Receiver<CopyClipEvent>, app: &AppHandle) {
    loop {
        let event = rx.recv().unwrap();
        match event {
            // update the clips in the tray menu
            CopyClipEvent::TrayUpdateEvent => {
                let clip_data = app.state::<ClipDataMutex>();
                let mut clip_data = clip_data.clip_data.lock().unwrap();
                let res = clip_data.update_tray(app);
                if res.is_err() {
                    // TODO: send a notification of the error, and panic the whole app
                    println!("error: {}", res.err().unwrap().message());
                }
            }
            // rebuild the tray menu
            CopyClipEvent::RebuildTrayMenuEvent => {
                // get number of clips to show from config
                let num = app.state::<ConfigMutex>();
                let num = num.config.lock().unwrap().clips_to_show;
                let res = app.tray_handle().set_menu(create_tray_menu(num));
                if res.is_err() {
                    println!("failed to set tray menu");
                    panic!("{}", res.err().unwrap().to_string());
                }
                // initial the tray
                send_tray_update_event(app);
            }
        }
    }
}
