/// the handler to monitor the change of the system clipboard
/// use the package provided by https://github.com/DoumanAsh/clipboard-master
/// Supported platforms
///   - Windows - uses dummy window to receive messages when clipboard changes;
///   - Linux - uses x11_clipboard (require to install xcb library); TODO add the requirement to the readme
///   - MacOS - uses polling via NSPasteboard::changeCount as there is no event notification.
/// 

use clipboard_master::{Master, ClipboardHandler, CallbackResult};
use tauri::{AppHandle, Manager, ClipboardManager};

use std::io;

use super::ClipDataMutex;

pub struct Handler<'a>{
      last_clip : String,
      app: &'a AppHandle,
}

impl ClipboardHandler for &mut Handler<'_> {
      fn on_clipboard_change(&mut self) -> CallbackResult {
            let clipboard_manager = self.app.clipboard_manager();
            let clip = clipboard_manager.read_text();
            if clip.is_err() {
                  return CallbackResult::StopWithError(io::Error::new(io::ErrorKind::Other, "error reading clipboard"));
            }
            let clip = clip.unwrap();
            if clip.is_none() {
                  return CallbackResult::StopWithError(io::Error::new(io::ErrorKind::Other, "error reading clipboard"));
            }
            let clip = clip.unwrap();
            if clip == self.last_clip {
                  return CallbackResult::Next;
            }
            self.last_clip = clip.clone();
            let clips = self.app.state::<ClipDataMutex>();
            let mut clip_data = clips.clip_data.lock().unwrap();
            let res = clip_data.new_clip(clip);
            if res.is_err() {
                  return CallbackResult::StopWithError(io::Error::new(io::ErrorKind::Other, "error: ".to_string() + &res.err().unwrap()));
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
      if last.is_some(){
            let last_t = last.unwrap();
            let last_t = (*last_t).clone();
            let t = clip_data.get_clip(last_t);
            if t.is_ok() {
            handler.last_clip = t.unwrap().text;
            }
      }
      drop(clip_data);
      drop(clips);

      let mut master = Master::new(&mut handler);
      master.run().unwrap();
}