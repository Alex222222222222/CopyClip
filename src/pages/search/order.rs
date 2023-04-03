use std::{
    fmt::Display,
    rc::Rc,
    sync::{Arc, Mutex},
};

use super::clip::Clip;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
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

pub fn sort_search_res(
    res: Arc<Mutex<Vec<Clip>>>,
    method: Rc<String>,
    // true for asc, false for desc
    order: bool,
) {
    let mut res = res.lock().unwrap();
    match method.as_str() {
        "time" => {
            res.sort_by(|a, b| {
                let res = a.timestamp.cmp(&b.timestamp);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "score" => {
            res.sort_by(|a, b| {
                let res = a.score.cmp(&b.score);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "id" => {
            res.sort_by(|a, b| {
                let res = a.id.cmp(&b.id);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "text" => {
            res.sort_by(|a, b| {
                let res = a.text.cmp(&b.text);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "len" => {
            res.sort_by(|a, b| {
                let res = a.len.cmp(&b.len);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        _ => {}
    }
}
