use yew::{function_component, html, use_effect_with, Html};

/// Display the search result
#[function_component(SearchDisplay)]
pub fn search() -> Html {
    let (search_constraints, _) = yewdux::use_store::<super::SearchConstraintStruct>();

    use_effect_with(search_constraints, move |search_constraints| {
        // TODO update search result
    });

    html! {
    <>
    </>
    }
}
