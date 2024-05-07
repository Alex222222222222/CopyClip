/// the handler to monitor the change of the system clipboard
/// use the package provided by https://github.com/DoumanAsh/clipboard-master
/// Supported platforms
///   - Windows - uses dummy window to receive messages when clipboard changes;
///   - Linux - uses x11_clipboard (require to install xcb library); TODO add the requirement to the readme
///   - MacOS - uses polling via NSPasteboard::changeCount as there is no event notification.
///
/// function "monitor_clip_board" monitor the system keyboard change
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use tauri::AppHandle;
use tauri_plugin_logging::panic_app;

use std::io;

use crate::event::{event_sender, CopyClipEvent};

use log::debug;

/// the handler for the system clipboard change
struct Handler<'a> {
    app: &'a AppHandle,
    first_time: bool,
}

/// the handler for the app clips data change
impl ClipboardHandler for &mut Handler<'_> {
    /// the handler for the system clipboard change
    /// this function is called when the system clipboard is changed
    /// try to read the clipboard, and if the clipboard is different from the last one, and current one update the app clips data
    fn on_clipboard_change(&mut self) -> CallbackResult {
        debug!("clipboard change");
        event_sender(self.app, CopyClipEvent::ClipboardChangeEvent);
        if self.first_time {
            self.first_time = false;
            event_sender(self.app, CopyClipEvent::ClipboardChangeEvent);
        }

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
    let mut handler = Handler {
        app,
        first_time: true,
    };

    let master = Master::new(&mut handler);
    let mut master = match master {
        Ok(master) => master,
        Err(err) => {
            panic_app(&format!("error creating clipboard master: {}", err));
            return;
        }
    };
    let res = master.run();
    if let Err(err) = res {
        panic_app(&format!("error running clipboard master: {}", err));
    }
}
