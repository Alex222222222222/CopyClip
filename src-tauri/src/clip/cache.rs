use tauri::{AppHandle, Manager};

use crate::config::ConfigMutex;

use super::{ClipDataMutex};

/// manage the cache of the clipboard
/// the daemon thread to manage clips cache
/// 

const CACHE_EXCEED_TIME: i64 = 3600;

pub fn cache_daemon(app: &AppHandle) {
      // the daemon thread to manage clips cache

      // get the clip per page config
      let config = app.state::<ConfigMutex>();
      let config = config.config.lock().unwrap();
      let clips_cache_basic = config.clips_to_show.clone() * 2;

      drop(config);

      loop {
            let clips = app.state::<ClipDataMutex>();
            let mut clips = clips.clip_data.lock().unwrap();

            // test if the 2*clips_per_page is loaded
            for i in 0..clips_cache_basic  {
                  let l = clips.clips.whole_list_of_ids.len();
                  let id = l as i64 - 1 - i ;
                  if id < 0 {
                        break;
                  }
                  let id = clips.clips.whole_list_of_ids.get(id as usize);
                  if id.is_none() {
                        break;
                  }
                  let id = id.unwrap();
                  let id = *id;

                  // test if id is in cache
                  let ok = clips.clips.cached_clips.contains_key(&id);
                  if !ok {
                        let res = clips.get_clip(id);
                        if res.is_err(){
                              // TODO log
                              break;
                        }
                  }
            }

            // get current timestamp
            let current_timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
            let mut ids_to_delete: Vec<i64> = vec![];
            for (id,clip) in &clips.clips.cached_clips {
                  if clip.add_timestamp - current_timestamp > CACHE_EXCEED_TIME {
                        let id = (*id).clone();
                        ids_to_delete.push(id);
                  }
            }

            for id in ids_to_delete {
                  clips.clips.cached_clips.remove(&id);
            }


            drop(clips);

            // sleep for half an hour
            std::thread::sleep(std::time::Duration::from_secs(1800));
      }
}