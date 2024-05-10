use std::{fmt::Display, rc::Rc, sync::Mutex};

use serde::{Deserialize, Serialize};

use super::clip::ClipWithSearchInfo;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum OrderOrder {
    Desc,
    Asc,
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

pub fn sort_search_res(
    res: Rc<Mutex<Vec<ClipWithSearchInfo>>>,
    method: OrderMethod,
    order: OrderOrder,
) {
    match order {
        OrderOrder::Asc => sort_search_res_asc(res, method),
        OrderOrder::Desc => sort_search_res_desc(res, method),
    }
}

fn sort_search_res_asc(res: Rc<Mutex<Vec<ClipWithSearchInfo>>>, method: OrderMethod) {
    let mut res = res.lock().unwrap();
    match method {
        OrderMethod::Time => {
            res.sort_by(|a, b| a.clip.timestamp.cmp(&b.clip.timestamp));
        }
        OrderMethod::FuzzyScore => {
            res.sort_by(|a, b| a.score.cmp(&b.score));
        }
        OrderMethod::Text => {
            res.sort_by(|a, b| a.clip.text.cmp(&b.clip.text));
        }
        OrderMethod::Size => {
            res.sort_by(|a, b| a.len.cmp(&b.len));
        }
    }
}

fn sort_search_res_desc(res: Rc<Mutex<Vec<ClipWithSearchInfo>>>, method: OrderMethod) {
    let mut res = res.lock().unwrap();
    match method {
        OrderMethod::Time => {
            res.sort_by(|a, b| b.clip.timestamp.cmp(&a.clip.timestamp));
        }
        OrderMethod::FuzzyScore => {
            res.sort_by(|a, b| b.score.cmp(&a.score));
        }
        OrderMethod::Text => {
            res.sort_by(|a, b| b.clip.text.cmp(&a.clip.text));
        }
        OrderMethod::Size => {
            res.sort_by(|a, b| b.len.cmp(&a.len));
        }
    }
}
