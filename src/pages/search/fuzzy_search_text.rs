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
    if text.len() > 500 {
        text = text[0..500].to_string();
    }
    let res = sublime_fuzzy::best_match(&data, &text);
    if res.is_none() {
        return html! {
            <td class="border border-gray-200">{text}</td>
        };
    }

    let res = res.unwrap();
    let mut start = 0;
    let mut end = 0;
    let text = res
        .continuous_matches()
        .into_iter()
        .map(|data: ContinuousMatch| {
            end = data.start();
            let before = &text[start..end];
            start = end + data.len();
            let mid = &text[end..start];
            html! {
                <>
                    {before} <span class="bg-yellow-300">{mid}</span>
                </>
            }
        })
        .collect::<Html>();

    html! {
        <td class="border border-gray-200">{text}</td>
    }
}
