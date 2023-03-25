use serde::{Serialize, Deserialize};

use super::clip::Clip;

#[derive(Serialize, Deserialize, PartialEq)]
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

pub fn sort_search_res(
    res: Vec<(i64, Clip)>,
    method: String,
    // true for asc, false for desc
    order: bool,
) -> Vec<(i64, Clip)> {
    let mut res = res;
    match method.to_lowercase().as_str() {
        "time" => {
            res.sort_by(|a, b| {
                let res = a.1.timestamp.cmp(&b.1.timestamp);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "score" => {
            res.sort_by(|a, b| {
                let res = a.1.score.cmp(&b.1.score);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "id" => {
            res.sort_by(|a, b| {
                let res = a.1.id.cmp(&b.1.id);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "text" => {
            res.sort_by(|a, b| {
                let res = a.1.text.cmp(&b.1.text);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        _ => {}
    }

    res
}