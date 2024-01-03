use super::clip_struct::Clip;
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::time::Duration;
use tauri::async_runtime::Mutex;

const MAX_CAPACITY: u64 = 32 * 1024 * 1024;
const INITIAL_CAPACITY: usize = 1000;
const TIME_TO_IDLE: Duration = Duration::from_secs(3600);

pub(super) struct ClipCache {
    cache: Mutex<Cache<i64, Clip>>,
}

pub(super) static CACHED_CLIP: Lazy<ClipCache> = Lazy::new(|| {
    let cache: Mutex<Cache<i64, Clip>> = Cache::builder()
        .weigher(|_, value: &Clip| -> u32 { value.text.len() as u32 })
        .max_capacity(MAX_CAPACITY)
        .initial_capacity(INITIAL_CAPACITY)
        .time_to_idle(TIME_TO_IDLE)
        .build()
        .into();
    ClipCache { cache }
});

impl ClipCache {
    pub(super) async fn get(&self, id: i64) -> Option<Clip> {
        let cache = self.cache.lock().await;
        cache.get(&id).clone()
    }

    pub(super) async fn insert(&self, id: i64, clip: Clip) {
        let cache = self.cache.lock().await;
        cache.insert(id, clip).await;
    }

    pub(super) async fn remove(&self, id: i64) {
        let cache = self.cache.lock().await;
        cache.invalidate(&id).await
    }

    pub(super) async fn update_favourite_state(&self, id: i64, favourite: bool) {
        // change the clip in the cache
        let clip = self.get(id).await;
        if let Some(mut clip) = clip {
            clip.favourite = favourite;
            // no need to wait for the result
            self.insert(id, clip).await;
        }
    }

    pub(super) async fn update_pinned_state(&self, id: i64, pinned: bool) {
        // change the clip in the cache
        let clip = self.get(id).await;
        if let Some(mut clip) = clip {
            clip.pinned = pinned;
            // no need to wait for the result
            self.insert(id, clip).await;
        }
    }
}
