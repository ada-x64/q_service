#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main, reason = "it is needed, actually")]
#![doc = include_str!("../README.md")]

/// Extends the Bevy [App](bevy_app::prelude::App) with service-related
/// functionality.
pub mod app;
/// Data types for services.
pub mod data;
/// Data types for service dependencies.
pub mod deps;
/// Lifecycle types.
pub mod lifecycle;
/// The main service resource.
pub mod service;
/// A macro for conveniently declaring and using services.
pub mod service_macro;
/// A declarative service specification used for adding new services to the app.
pub mod spec;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        app::*,
        data::*,
        lifecycle::{commands::*, events::*, hooks::*, run_conditions::*},
        service,
        service::*,
        spec::*,
    };
    #[cfg(feature = "derive")]
    pub use q_service_macros::*;
}
pub use paste;
