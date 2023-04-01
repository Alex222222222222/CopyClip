use std::collections::HashMap;

use once_cell::sync::Lazy;

pub static I18N: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(Languages::EnGb.get_language(), Languages::EnGb.to_label());
    map.insert(Languages::En.get_language(), Languages::En.to_label());
    map.insert(Languages::ZhCn.get_language(), Languages::ZhCn.to_label());
    map
});

pub enum Languages {
    EnGb,
    En,
    ZhCn,
}

impl Languages {
    pub fn get_language(&self) -> String {
        match self {
            Languages::EnGb => "en-GB".to_string(),
            Languages::En => "en".to_string(),
            Languages::ZhCn => "zh-CN".to_string(),
        }
    }

    pub fn to_label(&self) -> String {
        match self {
            Languages::EnGb => "English (UK)".to_string(),
            Languages::En => "English".to_string(),
            Languages::ZhCn => "中文".to_string(),
        }
    }
}
