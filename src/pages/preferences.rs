use yew::{function_component, html, Html};

use crate::components::{
    head_bar::HeadBar,
    preferences::{
        clips_per_page_config::ClipsPerPageConfig, dark_mode_switch::DarkModeSwitch,
        export_button::ExportButton, log_level_filter_config::LogLevelFilterConfig,
        max_clip_len_config::MaxClipLenConfig,
        clips_search_per_batch::SearchClipPerBatchConfig,
    },
};

#[function_component(Preferences)]
pub fn preferences() -> Html {
    html! {
        <div class="flex min-h-screen flex-col">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ "Preferences" }</h1>
            <div class="mx-5 my-2">
                <ClipsPerPageConfig></ClipsPerPageConfig>
                <br />
                <MaxClipLenConfig></MaxClipLenConfig>
                <br />
                // TODO add follow system theme
                <DarkModeSwitch></DarkModeSwitch>
                <br />
                <ExportButton></ExportButton>
            </div>

            <h2 class="text-center text-4xl m-0">{ "Advanced Config" }</h2>
            <div class="mx-5 my-2">
                <LogLevelFilterConfig></LogLevelFilterConfig>
                <br />
                <SearchClipPerBatchConfig></SearchClipPerBatchConfig>
            </div>
        </div>
    }
}
