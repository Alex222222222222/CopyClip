use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// the search_method module

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SearchMethod {
    /// search using sql like
    Normal,
    /// search using regex
    Regexp,
    /// search using fuzzy search
    Fuzzy,
}

impl Display for SearchMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMethod::Normal => f.write_str("normal"),
            SearchMethod::Regexp => f.write_str("regexp"),
            SearchMethod::Fuzzy => f.write_str("fuzzy"),
        }
    }
}

impl From<&str> for SearchMethod {
    fn from(s: &str) -> Self {
        match s {
            "normal" => SearchMethod::Normal,
            "regexp" => SearchMethod::Regexp,
            "fuzzy" => SearchMethod::Fuzzy,
            _ => SearchMethod::Normal,
        }
    }
}

impl From<String> for SearchMethod {
    fn from(value: String) -> Self {
        SearchMethod::from(value.as_str())
    }
}
