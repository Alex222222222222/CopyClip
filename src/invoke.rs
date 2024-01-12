use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogMsg {
    pub msg: String,
}

impl LogMsg {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl From<Option<String>> for LogMsg {
    fn from(msg: Option<String>) -> Self {
        LogMsg::new(match msg {
            Some(msg) => msg,
            None => "".to_string(),
        })
    }
}

impl From<JsValue> for LogMsg {
    fn from(msg: JsValue) -> Self {
        let msg = msg.as_string();
        msg.into()
    }
}
