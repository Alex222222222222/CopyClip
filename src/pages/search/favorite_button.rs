use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_state_eq, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

use crate::invoke::invoke;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Properties)]
pub struct FavoriteClipButtonProps {
    pub id: i64,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize)]
struct ChangeFavoriteClipArgs {
    pub id: i64,
    pub target: bool,
}

#[derive(PartialEq, Debug)]
enum IsFavorite {
    True,
    False,
}

impl IsFavorite {
    pub fn to_bool(&self) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
        }
    }

    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

#[function_component(FavoriteClipButton)]
pub fn favorite_clip_button(props: &FavoriteClipButtonProps) -> Html {
    let favorite = use_state_eq(|| IsFavorite::from_bool(props.is_favorite));
    let id = props.id;

    let favorite_1 = favorite.clone();
    let copy_clip_button_on_click = Callback::from(move |_| {
        let favorite_2 = favorite_1.clone();
        spawn_local(async move {
            let args = ChangeFavoriteClipArgs {
                id,
                target: !favorite_2.to_bool(),
            };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("change_favorite_clip", args).await;
            favorite_2.set(IsFavorite::from_bool(!favorite_2.to_bool()));
        });
    });

    let icon = if favorite.to_bool() {
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
