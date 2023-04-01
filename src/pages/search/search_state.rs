use yew::{function_component, html, Html, Properties};

use crate::components::loading::LoadingComponent;

/// search state of the search page
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum SearchState {
    NotStarted,
    Searching,
    Error(String),
    Finished,
}

impl SearchState {
    pub fn state(&self) -> SearchState {
        self.clone()
    }
}

/// convert the search state to html
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SearchStateHtmlProps {
    pub state: SearchState,
}

#[function_component(SearchStateHtml)]
pub fn search_state_html(props: &SearchStateHtmlProps) -> Html {
    let state = props.state.clone();
    match state {
        SearchState::NotStarted => html! {
            <></>
        },
        SearchState::Searching => html! {
            <>
                <LoadingComponent />
            </>
        },
        SearchState::Error(message) => html! {
            <label htmlFor="int-input-box" class=" text-xl">
                {message}
            </label>
        },
        SearchState::Finished => html! {
            <></>
        },
    }
}
