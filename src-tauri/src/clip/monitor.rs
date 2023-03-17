/// the handler to monitor the change of the system clipboard
/// use the package provided by https://github.com/DoumanAsh/clipboard-master
/// Supported platforms
///   - Windows - uses dummy window to receive messages when clipboard changes;
///   - Linux - uses x11_clipboard (require to install xcb library); TODO add the requirement to the readme
///   - MacOS - uses polling via NSPasteboard::changeCount as there is no event notification.
///
/// function "monitor" monitor the system keyboard change
/// function "clips_data_monitor" monitor the app clips data change, and trigger update of the tray
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use tauri::{AppHandle, ClipboardManager, Manager};

use std::io;

use super::ClipDataMutex;

pub struct Handler<'a> {
    last_clip: String,
    app: &'a AppHandle,
}

impl ClipboardHandler for &mut Handler<'_> {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let clipboard_manager = self.app.clipboard_manager();
        let clip = clipboard_manager.read_text();
        if clip.is_err() {
            return CallbackResult::StopWithError(io::Error::new(
                io::ErrorKind::Other,
                "error reading clipboard",
            ));
        }
        let clip = clip.unwrap();
        if clip.is_none() {
            return CallbackResult::StopWithError(io::Error::new(
                io::ErrorKind::Other,
                "error reading clipboard",
            ));
        }
        let clip = clip.unwrap();

        // if the clip is the same as the last one, do nothing
        if clip == self.last_clip {
            return CallbackResult::Next;
        }
        // if the clip is the same as the current one, do nothing
        // get the current_clip text
        let clips = self.app.state::<ClipDataMutex>();
        let mut clip_data = clips.clip_data.lock().unwrap();
        let current_clip = clip_data.get_current_clip();
        if current_clip.is_err() {
            return CallbackResult::StopWithError(io::Error::new(
                io::ErrorKind::Other,
                "error: ".to_string() + &current_clip.err().unwrap(),
            ));
        }
        let current_clip = current_clip.unwrap().text;
        if clip == current_clip {
            return CallbackResult::Next;
        }

        self.last_clip = clip.clone();
        let res = clip_data.new_clip(clip);
        if res.is_err() {
            return CallbackResult::StopWithError(io::Error::new(
                io::ErrorKind::Other,
                "error: ".to_string() + &res.err().unwrap(),
            ));
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, _error: io::Error) -> CallbackResult {
        // TODO: send a notification of the error, and panic the whole app
        CallbackResult::Next
    }
}

pub fn monitor(app: &AppHandle) {
    let mut handler = Handler {
        last_clip: String::new(),
        app,
    };

    // get last clip from database
    let clips = app.state::<ClipDataMutex>();
    let mut clip_data = clips.clip_data.lock().unwrap();
    let last = clip_data.clips.whole_list_of_ids.last();
    if let Some(last) = last {
        let last_t = last;
        let last_t = *last_t;
        let t = clip_data.get_clip(last_t);
        if let Ok(t) = t {
            // initially the last_clip is the last clip in the database
            handler.last_clip = t.text;
        }
    }
    drop(clip_data);

    let mut master = Master::new(&mut handler);
    master.run().unwrap();
}

pub fn clips_data_monitor(app: &AppHandle) {
    // monitor the whole list of ids len change
    let mut last_len = 0;

    // also monitor the current clip change
    let mut current_clip = 0;

    // also monitor the current page change
    let mut current_page = 0;
    loop {
        let clips = app.state::<ClipDataMutex>();
        let mut clip_data = clips.clip_data.lock().unwrap();
        if clip_data.clips.whole_list_of_ids.len() != last_len
            || clip_data.clips.current_clip != current_clip
            || clip_data.clips.current_page != current_page
        {
            last_len = clip_data.clips.whole_list_of_ids.len();
            current_clip = clip_data.clips.current_clip;
            current_page = clip_data.clips.current_page;
            let res = clip_data.update_tray(app);
            if res.is_err() {
                // TODO: send a notification of the error, and panic the whole app
                println!("error: {}", res.err().unwrap());
            }
        }
        drop(clip_data);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
