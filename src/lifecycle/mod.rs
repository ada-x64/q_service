//! This module is about the service lifecycle.
//!
//! Services go through a few distinct phases, as represented by the chart
//! below. ![service-lifecycle.png](TODO)
//!
//! When initializing a service, you can add [hooks](./hooks) to each lifecycle
//! phase. For example:
//!
//! ```rust
//! use bevy::prelude::*;
//! use q_service::prelude::*;
//!
//! service!(Example, (), ExampleError)
//!
//! let app = App::new();
//! app.add_service(
//!     EXAMPLE_SERVICE_SPEC
//!         .on_init(|| info!("This can be any system."));
//! )
//! ```
//!
//! You can also react to service state changes using events...
//!
//! ```rust, skip
//! app.add_observer(|trigger: Trigger<ExampleServiceEnabled>| {/*...*/})
//! ```
//!
//! ... and you can define run conditions.
//! ```rust, skip
//! app.add_systems(Update, (my_systems).run_if(service_enabled(ExampleService::handle())));
//! ```

/// Extends [Commands](bevy_ecs::prelude::Commands) with service functionality.
pub mod commands;
/// Events for interacting with services.
pub mod events;
/// Hooks used to intercept lifecycle stages.
pub mod hooks;
/// Run conditions for systems based on service state.
pub mod run_conditions;
