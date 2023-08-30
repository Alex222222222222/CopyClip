use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, Properties};
use yew_icons::{Icon, IconId};
use yewdux::prelude::Store;
use yewdux::prelude::use_store;

use crate::invoke::invoke;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub enum FavouriteFilter {
    All,
    Favourite,
    NotFavourite,
}

impl Default for FavouriteFilter {
    fn default() -> Self {
        Self::All
    }
}

impl FavouriteFilter {
    pub fn to_int(&self) -> i64 {
        match self {
            Self::All => -1,
            Self::Favourite => 1,
            Self::NotFavourite => 0,
        }
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct FavouriteClipButtonProps {
    pub id: i64,
    pub is_favourite: bool,
}

#[derive(Debug, Serialize)]
struct ChangeFavouriteClipArgs {
    pub id: i64,
    pub target: bool,
}

#[derive(PartialEq, Debug, Store, Clone, Eq, Default, Serialize, Deserialize)]
#[store(storage = "session")]
pub struct IsFavoriteID {
    pub content : HashSet<i64>,
}

#[function_component(FavouriteClipButton)]
pub fn favourite_clip_button(props: &FavouriteClipButtonProps) -> Html {
    let (favourite, dispatch) = use_store::<IsFavoriteID>();
    let id = props.id;

    let contain = favourite.content.contains(&id);
    if props.is_favourite != contain {
        dispatch.reduce_mut(|state| {
            if contain {
                state.content.insert(id);
            } else {
                state.content.remove(&id);
            }
        });
    }

    let favourite1 = favourite.clone();
    let copy_clip_button_on_click = Callback::from(move |_| {
        let favourite = favourite1.clone();
        let dispatch = dispatch.clone();
        spawn_local(async move {
            let args = ChangeFavouriteClipArgs {
                id,
                target: !favourite.content.contains(&id),
            };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("change_favourite_clip", args).await;
            dispatch.reduce_mut(|state| {
                if favourite.content.contains(&id) {
                    state.content.remove(&id);
                } else {
                    state.content.insert(id);
                }
            });
        });
    });

    let icon = if favourite.content.contains(&id) {
        IconId::BootstrapHeartFill
    } else {
        IconId::BootstrapHeart
    };

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold w-full"
                onclick={copy_clip_button_on_click}
            >
                <Icon icon_id={icon} class="mx-auto mt-0.5"/>
            </button>
        </td>
    }
}
