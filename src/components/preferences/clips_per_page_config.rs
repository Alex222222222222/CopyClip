use yew::{function_component, html, Html};

use super::int_config_template::IntConfigTemplate;

#[function_component(ClipsPerPageConfig)]
pub fn clips_per_page_config() -> Html {
    html! {
        <IntConfigTemplate
            label={"Clips per page"}
            default_value=20
            set_value_invoke={"set_per_page_data"}
            get_value_invoke={"get_per_page_data"}
        />
    }
}
