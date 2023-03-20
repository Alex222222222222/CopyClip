use std::sync::{mpsc::Sender, Mutex};

use tauri::{AppHandle, Manager};

use crate::clip::ClipDataMutex;

pub enum CopyClipEvent {
    TrayUpdateEvent,
}

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

pub fn event_daemon(rx: std::sync::mpsc::Receiver<CopyClipEvent>, app: &AppHandle) {
    loop {
        let event = rx.recv().unwrap();
        match event {
            CopyClipEvent::TrayUpdateEvent => {
                let clip_data = app.state::<ClipDataMutex>();
                let mut clip_data = clip_data.clip_data.lock().unwrap();
                let res = clip_data.update_tray(app);
                if res.is_err() {
                    // TODO: send a notification of the error, and panic the whole app
                    println!("error: {}", res.err().unwrap().message());
                }
            }
        }
    }
}
