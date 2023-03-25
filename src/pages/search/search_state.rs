use serde::{Deserialize, Serialize};
use yew::{function_component, html, Html, Properties};

/// search state of the search page
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

    pub fn is_err(&self) -> bool {
        match self {
            SearchState::Error(_) => true,
            _ => false,
        }
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
            <label htmlFor="int-input-box" class=" text-xl">
                {"Press search to start"}
            </label>
        },
        SearchState::Searching => html! {
            <label htmlFor="int-input-box" class=" text-xl">
                {"Searching"}
            </label>
        },
        SearchState::Error(message) => html! {
            <label htmlFor="int-input-box" class=" text-xl">
                {message}
            </label>
        },
        SearchState::Finished => html! {
            <label htmlFor="int-input-box" class=" text-xl">
                {"Search finished"}
            </label>
        },
    }
}
