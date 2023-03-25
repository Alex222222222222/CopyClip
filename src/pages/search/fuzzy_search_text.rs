use gloo_console::log;
use sublime_fuzzy::ContinuousMatch;
use yew::{function_component, html, Html, Properties};

#[derive(Clone, PartialEq, Properties)]
pub struct FuzzySearchTextProps {
    pub data: String,
    pub text: String,
}

/// fuzzy search text
#[function_component(FuzzySearchText)]
pub fn fuzzy_search_text(props: &FuzzySearchTextProps) -> Html {
    let data = props.data.clone();
    let mut text = props.text.clone();
    // if text.len() > 500, text = text[0..500].to_string() + "...";
    text.truncate(500);
    let res = sublime_fuzzy::best_match(&data, &text);
    if res.is_none() {
        return html! {
            <td class="border border-gray-200">{text}</td>
        };
    }

    let res = res.unwrap();
    let mut start = 0;
    let mut end = 0;
    log!("res: {:?}", text.clone());
    let text_data = res
        .continuous_matches()
        .map(|data: ContinuousMatch| {
            end = data.start();
            let res = try_get_uft8_code(&text, start, end);
            log!("start: {}, end: {}, len: {}", start, end, data.len());
            start = end + data.len();
            log!("start: {}, end: {}, len: {}", start, end, data.len());
            end = res.0;
            log!("start: {}, end: {}, len: {}", start, end, data.len());
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
    let start = start;
    let mut end = end;
    while end < text.len() {
        let res = text.get(start..end);
        if let Some(res) = res {
            return (end, res.to_string());
        } else {
            end += 1;
        }
    }

    (text.len() - 1, text[start..end].to_string())
}
