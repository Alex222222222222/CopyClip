use clip::TextSearchMethod;
use wasm_bindgen::JsCast;
use yew::{function_component, html, use_effect, Html, TargetCast};
use yew_icons::{Icon, IconId};

#[function_component(EnableFuzzySearchIcon)]
fn enable_fuzzy_search_icon() -> Html {
    let (search_constraints, search_dispatch) =
        yewdux::use_store::<super::SearchConstraintStruct>();
    let fuzzy_on_click = yew::Callback::from(move |_| {
        search_dispatch.reduce_mut(|state| {
            if state.text_search_method == TextSearchMethod::Fuzzy {
                state.text_search_method = TextSearchMethod::Contains;
            } else {
                state.text_search_method = TextSearchMethod::Fuzzy;
            }
        });
    });

    html! {
        <Icon class={if search_constraints.text_search_method == TextSearchMethod::Fuzzy {
                "p-1 my-1 mr-1 h-6 rounded-lg hover:cursor-pointer bg-blue-400"
            } else {
                "p-1 my-1 mr-1 h-6 rounded-lg hover:cursor-pointer text-black dark:text-white"
            }}
            icon_id={IconId::HeroiconsMiniSolidMagnifyingGlassPlus}
            title={t!("search.enable_fuzzy_search")}
            onclick={fuzzy_on_click}
        />
    }
}

#[function_component(EnableRegexSearchIcon)]
fn enable_regex_search_icon() -> Html {
    let (search_constraints, search_dispatch) =
        yewdux::use_store::<super::SearchConstraintStruct>();
    let regex_on_click = yew::Callback::from(move |_| {
        search_dispatch.reduce_mut(|state| {
            if state.text_search_method == TextSearchMethod::Regex {
                state.text_search_method = TextSearchMethod::Contains;
            } else {
                state.text_search_method = TextSearchMethod::Regex;
            }
        });
    });

    html! {
        <Icon class={if search_constraints.text_search_method == TextSearchMethod::Regex {
                "p-1 my-1 ml-1 h-6 rounded-lg hover:cursor-pointer bg-blue-400"
            } else {
                "p-1 my-1 ml-1 h-6 rounded-lg hover:cursor-pointer text-black dark:text-white"
            }}
            icon_id={IconId::LucideRegex}
            title={t!("search.enable_regex_search")}
            onclick={regex_on_click}
        />
    }
}

#[function_component(SearchInputBox)]
fn search_input_box() -> Html {
    // add event listener for keydown "/" to focus on search bar
    use_effect(move || {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let search_bar = document.get_element_by_id("search-bar-input").unwrap();
        let listen_key_board_event =
            wasm_bindgen::prelude::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if event.key() == "/" {
                    search_bar
                        .dyn_ref::<web_sys::HtmlInputElement>()
                        .unwrap()
                        .focus()
                        .unwrap();
                    event.prevent_default();
                }
            }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback(
                "keydown",
                listen_key_board_event.as_ref().unchecked_ref(),
            )
            .unwrap();

        // clean up
        move || {
            window
                .remove_event_listener_with_callback(
                    "keydown",
                    listen_key_board_event.as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    });

    let search_dispatch = yewdux::use_dispatch::<super::SearchConstraintStruct>();
    let search_dispatch_1 = search_dispatch.clone();
    let text_box_change_callback = yew::Callback::from(move |event: yew::Event| {
        let search_text = event
            .target_unchecked_into::<web_sys::HtmlInputElement>()
            .value();
        search_dispatch_1.reduce_mut(|state| {
            state.search_text = search_text;
        });
    });

    html! {
        <input class="bg-transparent text-black dark:text-white flex-grow"
            placeholder={t!("search.search_placeholder")}
            id="search-bar-input"
            onchange={text_box_change_callback}
        />
    }
}

#[function_component(ToggleDarkModeIcon)]
fn toggle_dark_mode_icon() -> Html {
    let (dark_mode_config, dark_mode_dispatch) =
        yewdux::use_store::<crate::route::DarkModeConfig>();

    let dark_mode_config_1 = dark_mode_config.clone();
    let toggle_dark_mode = yew::Callback::from(move |_| {
        let value = !dark_mode_config_1.is_dark;

        dark_mode_dispatch.set(crate::route::DarkModeConfig { is_dark: value });

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.document_element().unwrap();
        let class_list = element.class_list();
        if value {
            class_list.add_1("dark").unwrap();
        } else {
            class_list.remove_1("dark").unwrap();
        }
    });

    html! {
        <a class="h-12 text-black dark:text-white mx-2 hover:cursor-pointer"
            onclick={toggle_dark_mode}
        >
            if dark_mode_config.is_dark {
                <Icon class="my-3" icon_id={IconId::BootstrapMoonStarsFill}/>
            } else {
                <Icon class="my-3" icon_id={IconId::FontAwesomeSolidSun}/>
            }
        </a>
    }
}

#[function_component(SearchHeadBar)]
pub fn search() -> Html {
    let dark_mode_config = yewdux::use_store_value::<crate::route::DarkModeConfig>();

    html! {
        <div class="w-ful flex flex-row h-12">
            <div class="bg-gray-100 dark:bg-gray-900 border rounded-full px-2 m-2 flex flex-row h-8 text-lg w-32">
                if dark_mode_config.is_dark {
                    <img class="py-1 mr-1" src="/public/icons/icon.png" alt="search icon"
                        width="24" height="24"
                    />
                } else {
                    <img class="py-1 mr-1" src="/public/icons/icon-black.png" alt="search icon"
                        width="24" height="24"
                    />
                }
                <span class="text-black dark:text-white">{"CopyClip"}</span>
            </div>
            // search bar
            <div class=" border rounded-full h-8 my-2 bg-gray-100 flex-grow dark:bg-gray-900 flex flex-row">
                <Icon class="my-2 mx-2 h-4 text-black dark:text-white" icon_id={IconId::FontAwesomeSolidMagnifyingGlass}/>
                <SearchInputBox />
                // the hidden div is for the css to work, as tailwindcss does not support sibling selector
                <div class="p-1 m-1 h-4 bg-blue-400 text-black dark:text-white hidden rounded-lg" />
                <EnableRegexSearchIcon />
                <EnableFuzzySearchIcon />
            </div>
            // dark mode switch
            <ToggleDarkModeIcon />
            <Icon
                class="h-12 text-black dark:text-white mr-2 py-3 hover:cursor-pointer"
                icon_id={IconId::HeroiconsOutlineListBullet}
                title="Preferences"
                // TODO add preferences page
            />
        </div>
    }
}
