use std::collections::HashMap;

use yew::{function_component, html, Html};

use crate::components::preferences::option_choose_config_template::OptionChooseConfigTemplate;

#[function_component(LogLevelFilterConfig)]
pub fn log_level_filter_config() -> Html {
    html! {
        <OptionChooseConfigTemplate
            label={t!("preferences.log_level_filter").to_string()}
            get_value_invoke="get_log_level_filter"
            set_value_invoke="set_log_level_filter"
            default_value="info"
            option={
                let mut map = HashMap::new();
                map.insert("trace".to_string(), t!("log_level.trace").to_string());
                map.insert("debug".to_string(), t!("log_level.debug").to_string());
                map.insert("info".to_string(), t!("log_level.info").to_string());
                map.insert("warn".to_string(), t!("log_level.warn").to_string());
                map.insert("error".to_string(), t!("log_level.error").to_string());
                map.insert("off".to_string(), t!("log_level.off").to_string());
                map
            }
        />
    }
}
