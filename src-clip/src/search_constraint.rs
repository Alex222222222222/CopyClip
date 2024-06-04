use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Limit the number of results
    Limit(usize),
}
