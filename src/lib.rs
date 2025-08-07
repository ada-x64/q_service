#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main, reason = "it is needed, actually")]
#![doc = include_str!("../README.md")]

/// Extensions to [App](bevy_app::App).
pub mod app;
mod data;
/// Dependency management.
pub mod deps;
pub(crate) mod graph;
/// Service lifecycle functions.
pub mod lifecycle;
/// [Conditions](bevy_ecs::schedule::Condition) for service scoping.
pub mod run_conditions;
/// The [ServiceScope](crate::prelude::ServiceScope) struct.
pub mod scope;
/// The inner [ServiceData](crate::prelude::ServiceData) implementation.
pub mod service_data;
/// The user-facing [Service](crate::prelude::Service) trait
pub mod service_trait;
mod spec;
/// [SystemParams](bevy_ecs::system::SystemParam) for [Services](crate::prelude::Service).
pub mod system_params;
/// Asynchronous tasks forked from [q_tasks](https://docs.io/q_tasks)
pub mod tasks;
/// Extensions to [World](bevy_ecs::prelude::World).
pub mod world;

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::{
        app::*,
        data::*,
        deps::*,
        graph::{DependencyGraph, NodeId},
        lifecycle::{commands::*, events::*, hooks::*},
        run_conditions::*,
        scope::*,
        service_data::*,
        service_trait::*,
        system_params::*,
        tasks::*,
        world::*,
    };
}

// for use in macros
#[doc(hidden)]
pub use bevy_derive;
#[doc(hidden)]
pub use bevy_ecs;
#[doc(hidden)]
pub use paste;
