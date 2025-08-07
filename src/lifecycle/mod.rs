#![doc=include_str!("./doc.md")]

/// Events used to update the service lifecycle.
pub mod commands;
/// Events for reacting to service changes.
pub mod events;
/// Hooks used to intercept lifecycle stages.
pub mod hooks;
