/// the handler to monitor the change of the system clipboard
/// use the package provided by https://github.com/DoumanAsh/clipboard-master
/// Supported platforms
///   - Windows - uses dummy window to receive messages when clipboard changes;
///   - Linux - uses x11_clipboard (require to install xcb library); TODO add the requirement to the readme
///   - MacOS - uses polling via NSPasteboard::changeCount as there is no event notification.
///
/// function "monitor_clip_board" monitor the system keyboard change
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use tauri::{AppHandle, Manager};

use std::io;

use crate::{
    event::{CopyClipEvent, EventSender},
    log::panic_app,
};

use log::{debug, error};

/// the handler for the system clipboard change
struct Handler<'a> {
    app: &'a AppHandle,
}

/// the handler for the app clips data change
impl ClipboardHandler for &mut Handler<'_> {
    /// the handler for the system clipboard change
    /// this function is called when the system clipboard is changed
    /// try to read the clipboard, and if the clipboard is different from the last one, and current one update the app clips data
    fn on_clipboard_change(&mut self) -> CallbackResult {
        debug!("clipboard change");
        let tx = self.app.state::<EventSender>();
        let tx = tx.tx.clone();
        tauri::async_runtime::spawn(async move {
            let res = tx.send(CopyClipEvent::ClipboardChangeEvent).await;
            if let Err(err) = res {
                error!("Failed to send event, error: {}", err);
                panic_app(&format!("Failed to send event, error: {}", err));
            }
        });

        CallbackResult::Next
    }

    /// the handler for the error
    /// send a notification of the error, and panic the whole app
    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        panic_app(&format!("error reading clipboard: {}", error));
        CallbackResult::Next
    }
}

/// monitor the app clips data change, and trigger update of the tray
pub async fn monitor_clip_board(app: &AppHandle) {
    let mut handler = Handler { app };

    let mut master = Master::new(&mut handler);
    master.run().unwrap();
}
