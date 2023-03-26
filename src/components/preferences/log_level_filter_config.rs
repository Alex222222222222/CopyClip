use std::collections::HashMap;

use yew::{function_component, html, Html};

use crate::components::preferences::option_choose_config_template::OptionChooseConfigTemplate;

#[function_component(LogLevelFilterConfig)]
pub fn log_level_filter_config() -> Html {
    html! {
        <OptionChooseConfigTemplate
            label="Log Level Filter"
            get_value_invoke="get_log_level_filter"
            set_value_invoke="set_log_level_filter"
            default_value="info"
            option={
                let mut map = HashMap::new();
                map.insert("trace".to_string(), "Trace".to_string());
                map.insert("debug".to_string(), "Debug".to_string());
                map.insert("info".to_string(), "Info".to_string());
                map.insert("warn".to_string(), "Warn".to_string());
                map.insert("error".to_string(), "Error".to_string());
                map.insert("off".to_string(), "Off".to_string());
                map
            }
        />
    }
}
