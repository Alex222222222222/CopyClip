use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_state_eq, Callback, Html, Properties};
use yew_icons::{Icon, IconId};

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

#[derive(PartialEq, Debug)]
enum IsFavourite {
    True,
    False,
}

impl IsFavourite {
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

#[function_component(FavouriteClipButton)]
pub fn favourite_clip_button(props: &FavouriteClipButtonProps) -> Html {
    let favourite = use_state_eq(|| IsFavourite::from_bool(props.is_favourite));
    let id = props.id;

    let favourite_1 = favourite.clone();
    let copy_clip_button_on_click = Callback::from(move |_| {
        let favourite_2 = favourite_1.clone();
        spawn_local(async move {
            let args = ChangeFavouriteClipArgs {
                id,
                target: !favourite_2.to_bool(),
            };
            let args = serde_wasm_bindgen::to_value(&args).unwrap();
            invoke("change_favourite_clip", args).await;
            favourite_2.set(IsFavourite::from_bool(!favourite_2.to_bool()));
        });
    });

    let icon = if favourite.to_bool() {
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
