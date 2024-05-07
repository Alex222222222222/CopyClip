use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    pub fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub fn trace(msg: String) {
    let args = to_value(&LogMsg { msg }).unwrap();
    invoke("'plugin:logging|trace'", args);
}

pub fn info(msg: String) {
    let args = to_value(&LogMsg { msg }).unwrap();
    invoke("'plugin:logging|info'", args);
}

pub fn debug(msg: String) {
    let args = to_value(&LogMsg { msg }).unwrap();
    invoke("'plugin:logging|debug'", args);
}

pub fn error(msg: String) {
    let args = to_value(&LogMsg { msg }).unwrap();
    invoke("'plugin:logging|error'", args);
}

pub fn warn(msg: String) {
    let args = to_value(&LogMsg { msg }).unwrap();
    invoke("'plugin:logging|warn'", args);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LogMsg {
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
