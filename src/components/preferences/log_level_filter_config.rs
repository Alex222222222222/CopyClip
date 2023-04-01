use std::collections::HashMap;

use yew::{function_component, html, Html};

use crate::components::preferences::option_choose_config_template::OptionChooseConfigTemplate;

#[function_component(LogLevelFilterConfig)]
pub fn log_level_filter_config() -> Html {
    html! {
        <OptionChooseConfigTemplate
            label={t!("preferences.log_level_filter")}
            get_value_invoke="get_log_level_filter"
            set_value_invoke="set_log_level_filter"
            default_value="info"
            option={
                let mut map = HashMap::new();
                map.insert("trace".to_string(), t!("log_level.trace"));
                map.insert("debug".to_string(), t!("log_level.debug"));
                map.insert("info".to_string(), t!("log_level.info"));
                map.insert("warn".to_string(), t!("log_level.warn"));
                map.insert("error".to_string(), t!("log_level.error"));
                map.insert("off".to_string(), t!("log_level.off"));
                map
            }
        />
    }
}
