use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub struct  ConfigMutex{
      pub config: Mutex<Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub clips_to_show: usize,
}