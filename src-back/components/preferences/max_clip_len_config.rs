use yew::{function_component, html, Html};

use super::int_config_template::IntConfigTemplate;

#[function_component(MaxClipLenConfig)]
pub fn max_clip_len_config() -> Html {
    html! {
        <IntConfigTemplate
            label={"preferences.max_clip_len"}
            default_value=20
            set_value_invoke={"set_max_clip_len"}
            get_value_invoke={"get_max_clip_len"}
        />
    }
}
