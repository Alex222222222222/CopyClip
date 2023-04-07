use std::{fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};
use yew::{function_component, html, Html, Properties};

use crate::components::loading::LoadingComponent;

use super::SearchFullArgs;

/// search state of the search page
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SearchState {
    NotStarted,
    Searching,
    Error(String),
    Finished,
}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SearchState::NotStarted, SearchState::NotStarted) => true,
            (SearchState::Searching, SearchState::Searching) => true,
            (SearchState::Error(m1), SearchState::Error(m2)) => m1 == m2,
            (SearchState::Finished, SearchState::Finished) => true,
            _ => false,
        }
    }
}

impl Display for SearchState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchState::NotStarted => write!(f, "NotStarted"),
            SearchState::Searching => write!(f, "Searching"),
            SearchState::Error(message) => write!(f, "Error: {}", message),
            SearchState::Finished => write!(f, "Finished"),
        }
    }
}

impl SearchState {
    pub fn html(&self) -> Html {
        match self {
            SearchState::NotStarted => html! {
                <>
                    {"test"}
                </>
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
                <>
                    {"test1"}
                </>
            },
        }
    }
}

/// convert the search state to html
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SearchStateHtmlProps {
    pub state: Rc<SearchFullArgs>,
}

#[function_component(SearchStateHtml)]
pub fn search_state_html(props: &SearchStateHtmlProps) -> Html {
    props.state.search_state.html()
}
