use yew::{function_component, html, Html};

use crate::components::{
    head_bar::HeadBar,
    preferences::{
        clips_per_page_config::ClipsPerPageConfig,
        clips_search_per_batch::SearchClipPerBatchConfig, dark_mode_switch::DarkModeSwitch,
        export_button::ExportButton, language_config::LanguagesConfig,
        log_level_filter_config::LogLevelFilterConfig, max_clip_len_config::MaxClipLenConfig,
    },
};

#[function_component(Preferences)]
pub fn preferences() -> Html {
    html! {
        <div class="flex min-h-screen flex-col">
            <HeadBar></HeadBar>
            <h1 class="text-center text-6xl m-0">{ t!("preferences.title") }</h1>
            <div class="mx-5 my-2">
                <ClipsPerPageConfig></ClipsPerPageConfig>
                <br />
                <MaxClipLenConfig></MaxClipLenConfig>
                <br />
                // TODO add follow system theme
                <DarkModeSwitch></DarkModeSwitch>
                <br />
                <LanguagesConfig></LanguagesConfig>
                // TODO i18n
                <ExportButton></ExportButton>
            </div>

            <h2 class="text-center text-4xl m-0">{ t!("preferences.advanced_title") }</h2>
            <div class="mx-5 my-2">
                <LogLevelFilterConfig></LogLevelFilterConfig>
                <br />
                <SearchClipPerBatchConfig></SearchClipPerBatchConfig>
            </div>
        </div>
    }
}
