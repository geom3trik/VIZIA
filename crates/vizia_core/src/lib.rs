#![allow(clippy::type_complexity)]
// To allow a match syntax in event handlers with one event variant
#![allow(clippy::single_match)]
// To allow enum names with the same prefix
#![allow(clippy::enum_variant_names)]
#![allow(clippy::only_used_in_recursion)]

extern crate self as vizia;

pub(crate) mod accessibility;
pub mod animation;
pub mod binding;
#[doc(hidden)]
pub(crate) mod cache;
pub mod context;
#[doc(hidden)]
pub(crate) mod entity;
pub mod environment;
pub mod events;
pub mod input;
pub mod layout;
pub mod localization;
pub mod model;
pub mod modifiers;
pub mod resource;
pub mod style;
pub(crate) mod systems;
pub(crate) mod text;
#[doc(hidden)]
pub mod tree;
pub mod util;
pub mod view;
pub mod views;
pub mod window;

mod storage;

/// Contains types and functions used for custom drawing within views. This is a re-export of [skia-safe](https://github.com/rust-skia/rust-skia).
pub mod skia {
    pub use skia_safe::*;
}

/// A collection of built-in SVG icons.
pub mod icons;

#[doc(hidden)]
pub mod backend {
    pub use super::accessibility::IntoNode;
    pub use super::context::backend::BackendContext;
    pub use vizia_window::WindowDescription;
}

/// Members which we recommend you wildcard-import.
#[doc(hidden)]
pub mod prelude {
    pub use super::binding::{
        Binding, Data, Index, Lens, LensExt, LensValue, Map, MapRef, Res, ResGet, StaticLens, Then,
        UnwrapLens, Wrapper,
    };

    pub use super::impl_res_simple;

    pub use crate::model::Model;

    pub use super::animation::{Animation, AnimationBuilder, KeyframeBuilder};
    pub use super::context::{
        AccessContext, AccessNode, Context, ContextProxy, DataContext, DrawContext, EmitContext,
        EventContext, ProxyEmitError, WindowState,
    };
    pub use super::entity::Entity;
    pub use super::environment::{AppTheme, Environment, EnvironmentEvent, ThemeMode};
    pub use super::events::{Event, Propagation, Timer, TimerAction};
    pub use super::include_style;
    pub use super::input::{Keymap, KeymapEntry, KeymapEvent};
    pub use super::layout::{BoundingBox, GeoChanged};
    pub use super::localization::{Localized, ToStringLocalized};
    pub use super::modifiers::{
        AbilityModifiers, AccessibilityModifiers, ActionModifiers, LayoutModifiers,
        LinearGradientBuilder, ShadowBuilder, StyleModifiers, TextModifiers,
    };
    pub use super::resource::{ImageId, ImageRetentionPolicy};
    pub use super::util::{IntoCssStr, CSS};
    pub use super::view::{Handle, View};
    pub use super::views::*;
    pub use super::window::{DropData, WindowEvent};
    pub use accesskit::{Action, DefaultActionVerb, Live, Role};
    pub use skia_safe::Canvas;
    pub use vizia_derive::{Data, Lens};
    pub use vizia_id::GenerationalId;
    pub use vizia_input::{Code, Key, KeyChord, Modifiers, MouseButton, MouseButtonState};
    pub use vizia_storage::{Tree, TreeExt};
    pub use vizia_window::{WindowButtons, WindowPosition, WindowSize};

    pub use super::style::*;

    pub use morphorm::Units::*;
    pub use morphorm::{LayoutType, PositionType, Units};
    pub use unic_langid::{langid, LanguageIdentifier};
    pub use web_time::{Duration, Instant};
}
