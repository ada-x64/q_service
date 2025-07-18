#![doc=include_str!("./doc.md")]

/// Extends [Commands](bevy_ecs::prelude::Commands) with service functionality.
pub mod commands;
/// Events for interacting with services.
pub mod events;
/// Hooks used to intercept lifecycle stages.
pub mod hooks;
/// Run conditions for systems based on service state.
pub mod run_conditions;
