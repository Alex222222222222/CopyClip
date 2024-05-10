use std::sync::Arc;

use sublime_fuzzy::ContinuousMatch;
use yew::{function_component, html, Html, Properties};

use super::search_method::SearchMethod;

#[derive(PartialEq, Properties)]
pub struct FuzzySearchTextProps {
    pub data: Arc<String>,
    pub text: Arc<String>,
}

#[derive(PartialEq, Properties)]
pub struct RegexpSearchTextProps {
    pub data: Arc<String>,
    pub text: Arc<String>,
}

#[derive(PartialEq, Properties)]
pub struct SearchTextProps {
    pub data: Arc<String>,
    pub text: Arc<String>,
    pub search_method: SearchMethod,
}

/// search text
#[function_component(SearchText)]
pub fn search_text(props: &SearchTextProps) -> Html {
    if props.search_method != SearchMethod::Regexp {
        html! {
            <FuzzySearchText data={props.data.clone()} text={props.text.clone()} />
        }
    } else {
        html! {
            <RegexpSearchText data={props.data.clone()} text={props.text.clone()} />
        }
    }
}

/// regexp search text
#[function_component(RegexpSearchText)]
pub fn regexp_search_text(props: &RegexpSearchTextProps) -> Html {
    let data = props.data.clone();
    let text = props.text.clone();
    let re = regex::Regex::new(&data);
    if re.is_err() {
        return html! {
            <td class="border border-gray-200">{text}</td>
        };
    }
    let re = re.unwrap();

    // if text.len() > 500, text = text[0..500].to_string() + "...";
    let text = try_get_uft8_code(&text, 0, 500).1;

    let mut start: usize = 0;
    let mut end: usize = 0;
    let text_data = re
        .find_iter(&text)
        .map(|m| {
            let location_res = (m.start(), m.end());
            let before = try_get_uft8_code(&text, end, location_res.0);
            start = before.0;
            let mid = try_get_uft8_code(&text, start, location_res.1);
            end = mid.0;

            html! {
                <>
                    {before.1}
                    <span class="bg-yellow-300">{mid.1}</span>
                </>
            }
        })
        .collect::<Html>();

    let end_text = try_get_uft8_code(&text, end, text.len());
    let text_data = html! {
        <>
            {text_data}
            {end_text.1}
        </>
    };

    html! {
        <td class="border border-gray-200">{text_data}</td>
    }
}

/// fuzzy search text
#[function_component(FuzzySearchText)]
pub fn fuzzy_search_text(props: &FuzzySearchTextProps) -> Html {
    let data = props.data.clone();
    if data.is_empty() {
        return html! {
            <td class="border border-gray-200">{props.text.clone()}</td>
        };
    }

    let text = props.text.clone();
    // if text.len() > 500, text = text[0..500].to_string() + "...";
    let text = try_get_uft8_code(&text, 0, 500).1;
    let res = sublime_fuzzy::best_match(&data, &text);
    if res.is_none() {
        return html! {
            <td class="border border-gray-200">{text}</td>
        };
    }

    let res = res.unwrap();
    let mut start = 0;
    let mut end = 0;
    let text_data = res
        .continuous_matches()
        .map(|data: ContinuousMatch| {
            end = data.start();
            let res = try_get_uft8_code(&text, start, end);
            start = end + data.len();
            end = res.0;
            let before = res.1;
            let res = try_get_uft8_code(&text, end, start);
            start = res.0;
            let mid = res.1;
            html! {
                <>
                    {before} <span class="bg-yellow-300">{mid}</span>
                </>
            }
        })
        .collect::<Html>();

    let text_data = html! {
        <>
            {text_data}
            {try_get_uft8_code(&text, start, text.len()).1}
        </>
    };

    html! {
        <td class="border border-gray-200">{text_data}</td>
    }
}

fn try_get_uft8_code(text: &str, start: usize, end: usize) -> (usize, String) {
    let mut end = end;
    while end < text.len() {
        let res = text.get(start..end);
        if let Some(res) = res {
            return (end, res.to_string());
        } else {
            end += 1;
        }
    }

    (text.len(), text[start..text.len()].to_string())
}
