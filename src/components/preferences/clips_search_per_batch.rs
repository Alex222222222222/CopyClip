use yew::{function_component, html, Html};

use super::int_config_template::IntConfigTemplate;

#[function_component(SearchClipPerBatchConfig)]
pub fn search_clip_per_batch_config() -> Html {
    html! {
        <IntConfigTemplate
            label={"Search Clip per Batch"}
            default_value=2
            set_value_invoke={"set_search_clip_per_batch"}
            get_value_invoke={"get_search_clip_per_batch"}
        />
    }
}
