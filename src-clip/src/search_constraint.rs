use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum SearchConstraint {
    /// Search for the text that contains the given text
    TextContains(String),
    /// Search for the text that match the regex
    TextRegex(String),
    /// Search for the text that match the fuzzy search
    TextFuzzy(String),
    /// Timestamp that is greater than the given timestamp
    TimestampGreaterThan(i64),
    /// Timestamp that is less than the given timestamp
    TimestampLessThan(i64),
    /// Has the given label
    HasLabel(String),
    /// Does not have the given label
    NotHasLabel(String),
    /// Limit the number of results
    Limit(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextSearchMethod {
    Contains,
    Regex,
    Fuzzy,
}
