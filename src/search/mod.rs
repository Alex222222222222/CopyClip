mod search_head_bar;
mod search_labels_bar;
mod search_display;

use std::collections::HashSet;

use clip::TextSearchMethod;
use search_head_bar::SearchHeadBar;
use search_labels_bar::SearchLabelsBar;
use yew::{function_component, html, Html};

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, yewdux::Store)]
#[store(storage = "local", storage_tab_sync)]
struct SearchConstraintStruct {
    search_text: String,
    neutral_label: HashSet<String>,
    exclude_label: HashSet<String>,
    text_search_method: TextSearchMethod,
}

impl Default for SearchConstraintStruct {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            neutral_label: HashSet::new(),
            exclude_label: HashSet::new(),
            text_search_method: TextSearchMethod::Contains,
        }
    }
}

/*
impl SearchConstraintStruct {
    fn to_search_constraints(&self) -> Vec<SearchConstraint> {
        let mut search_constraints = Vec::new();
        if !self.search_text.is_empty() {
            match self.text_search_method {
                TextSearchMethod::Contains => {
                    search_constraints
                        .push(SearchConstraint::TextContains(self.search_text.clone()));
                }
                TextSearchMethod::Regex => {
                    search_constraints.push(SearchConstraint::TextRegex(self.search_text.clone()));
                }
                TextSearchMethod::Fuzzy => {
                    search_constraints.push(SearchConstraint::TextFuzzy(self.search_text.clone()));
                }
            }
        }
        for label in &self.label {
            search_constraints.push(SearchConstraint::HasLabel(label.clone()));
        }
        search_constraints
    }
}
*/

#[function_component(Search)]
pub fn search() -> Html {
    html! {
        <div class="h-full w-full">
            <SearchHeadBar />
            <SearchLabelsBar />
        </div>
    }
}
