use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::pages::invoke;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum FavoriteFilter {
    All,
    Favorite,
    NotFavorite,
}

impl Default for FavoriteFilter {
    fn default() -> Self {
        Self::All
    }
}

impl FavoriteFilter {
    pub fn to_int(&self) -> i64 {
        match self {
            Self::All => -1,
            Self::Favorite => 1,
            Self::NotFavorite => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct FavoriteClipButtonProps {
    pub id: i64,
    pub is_favorite: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct ChangeFavoriteClipArgs {
    pub id: i64,
    pub target: bool,
}

#[function_component(FavoriteClipButton)]
pub fn favorite_clip_button(props: &FavoriteClipButtonProps) -> Html {
    let id = props.id;
    let is_favorite = props.is_favorite;
    let copy_clip_button_on_click = Callback::from(move |_| {
        spawn_local(async move {
            let args = ChangeFavoriteClipArgs {
                id,
                target: !is_favorite,
            };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("change_favorite_clip", args).await;
        });
    });

    let icon = if props.is_favorite {
        IconId::BootstrapHeartFill
    } else {
        IconId::BootstrapHeart
    };

    html! {
        <td class="border border-gray-200">
            <button
                class="font-bold"
                onclick={copy_clip_button_on_click}
            >
                <Icon icon_id={icon}/>
            </button>
        </td>
    }
}
