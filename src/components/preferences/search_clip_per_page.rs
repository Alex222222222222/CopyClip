use yew::{function_component, html, Html};

use super::int_config_template::IntConfigTemplate;

#[function_component(SearchClipPerPageConfig)]
pub fn search_clip_per_page_config() -> Html {
    html! {
        <IntConfigTemplate
            label={"Search clip per page"}
            default_value=20
            set_value_invoke={"set_search_clip_per_page"}
            get_value_invoke={"get_search_clip_per_page"}
        />
    }
}
