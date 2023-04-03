use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use super::clip::Clip;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum OrderOrder {
    Desc,
    Asc,
}

impl OrderOrder {
    pub fn to_bool(&self) -> bool {
        match self {
            OrderOrder::Desc => false,
            OrderOrder::Asc => true,
        }
    }
}

impl Display for OrderOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderOrder::Desc => f.write_str("desc"),
            OrderOrder::Asc => f.write_str("asc"),
        }
    }
}

/// the order method module
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum OrderMethod {
    /// order by fuzzy score
    FuzzyScore,
    /// order by id
    Id,
    /// order by size
    Size,
    /// order by text
    Text,
    /// order by time
    Time,
}

impl Display for OrderMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderMethod::FuzzyScore => write!(f, "fuzzy_score"),
            OrderMethod::Id => write!(f, "id"),
            OrderMethod::Size => write!(f, "size"),
            OrderMethod::Text => write!(f, "text"),
            OrderMethod::Time => write!(f, "time"),
        }
    }
}

impl From<&str> for OrderMethod {
    fn from(s: &str) -> Self {
        match s {
            "fuzzy_score" => OrderMethod::FuzzyScore,
            "id" => OrderMethod::Id,
            "size" => OrderMethod::Size,
            "text" => OrderMethod::Text,
            "time" => OrderMethod::Time,
            _ => panic!("unknown order method"),
        }
    }
}

impl From<String> for OrderMethod {
    fn from(s: String) -> Self {
        OrderMethod::from(s.as_str())
    }
}

pub fn sort_search_res(res: Arc<Mutex<Vec<Clip>>>, method: OrderMethod, order: OrderOrder) {
    match order {
        OrderOrder::Asc => sort_search_res_asc(res, method),
        OrderOrder::Desc => sort_search_res_desc(res, method),
    }
}

fn sort_search_res_asc(res: Arc<Mutex<Vec<Clip>>>, method: OrderMethod) {
    let mut res = res.lock().unwrap();
    match method {
        OrderMethod::Time => {
            res.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        OrderMethod::FuzzyScore => {
            res.sort_by(|a, b| a.score.cmp(&b.score));
        }
        OrderMethod::Id => {
            res.sort_by(|a, b| a.id.cmp(&b.id));
        }
        OrderMethod::Text => {
            res.sort_by(|a, b| a.text.cmp(&b.text));
        }
        OrderMethod::Size => {
            res.sort_by(|a, b| a.len.cmp(&b.len));
        }
    }
}

fn sort_search_res_desc(res: Arc<Mutex<Vec<Clip>>>, method: OrderMethod) {
    let mut res = res.lock().unwrap();
    match method {
        OrderMethod::Time => {
            res.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
        OrderMethod::FuzzyScore => {
            res.sort_by(|a, b| b.score.cmp(&a.score));
        }
        OrderMethod::Id => {
            res.sort_by(|a, b| b.id.cmp(&a.id));
        }
        OrderMethod::Text => {
            res.sort_by(|a, b| b.text.cmp(&a.text));
        }
        OrderMethod::Size => {
            res.sort_by(|a, b| b.len.cmp(&a.len));
        }
    }
}
