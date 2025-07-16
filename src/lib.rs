#![warn(missing_docs)]
//! This crate aims to bring the service model to Bevy.
//!
//! Bevy's ECS is like an operating system's process scheduler. It takes systems
//! (processes) that operate on data (files) and schedules them appropriately.
//! Well, if we have an OS analogue, we need an analogue for services.
//!
//! This crate is loosely modelled after [systemd](https://systemd.io).
//! It doesn't manage PID1, but it does manage user services, including those
//! necessary for startup.
//!
//! Services are resources which have built-in state, associated data, and
//! managed dependencies. This crate extends the Bevy ECS to include this
//! functionality. Because it is an engine extension, _there is no associated
//! plugin._ Simply import the prelude and you're good to go.
//!
//! I have tried to make this documentation intuitive to explore in your IDE,
//! assuming you're using rust-analyzer or a similar LSP. You might get more out
//! of reading it as you use it rather than reading it all ahead of time.
//!
//! ## Example usage
//! ```rust, skip
//! use bevy::prelude::*;
//! use q_service::prelude::*;
//! use thiserror::Error;
//!
//! // First, you need to define your service variables.
//! // Services are uniquely determined by three types:
//! // ServiceLabel, ServiceError, and ServiceData.
//! // ServiceLabel will be defined for you when you declare
//! // the service with `service!`, but you'll have to manually
//! // define your data and error types.
//!
//! #[derive(ServiceError, Error, Debug, Clone, PartialEq)]
//! enum MyError {}
//!
//! // If your service doesn't need any data, you can just pass in ().
//! #[derive(ServiceData, Debug, Clone, PartialEq)]
//! struct MyData {}
//!
//! // Declare the service!
//! // This will create a bunch of useful aliases and the
//! // ServiceLabel type.
//! service!(MyService, MyData, MyError);
//!
//! // Next, add the service to the application.
//! fn doit() {
//!     let mut app = App::new();
//!     // We use a ServiceSpec to declaratively define the behavior of the service.
//!     app.add_service(
//!         MyService::default_spec()
//!             // This service will initialize in the Startup schedule.
//!             // By default, services are lazily initialized whenever they are
//!             // enabled.
//!             .is_startup(true)
//!             // This service has some dependencies!
//!             // Before it initializes, it will initialize all of its
//!             // dependencies, and their dependencies. Dependencies
//!             // are usually other services, but they can be anything which
//!             // implements the `IsServiceDep` trait.
//!             .with_deps(vec![
//!                 // Service handles provide a convenient way to refer to
//!                 // services without passing them around.
//!                 // They're zero-sized types which act as a shorthand for the
//!                 // service's type specification.
//!                 MyOtherService::handle(),
//!                 // This might be an asset, a resource, some random data...
//!                 MyNonServiceDep,
//!             ])
//!             // Services can hold arbitrary data types. This doesn't affect their
//!             // lifecycle or dependencies at all.
//!             .with_data(MyData {
//!                 /* ... */
//!             })
//!             // You can also define hooks for the service lifecycle.
//!             // There are five main lifecycle events.
//!             // The first is initialization.
//!             .on_init(|world: &mut World| -> Result<bool, MyError> {
//!                 // This can be any system function, exclusive or otherwise.
//!                 // It just has to have the right return variable.
//!                 // Initialization can proceed to enable or disable the
//!                 // service, depending on the return value.
//!                 Ok(true)
//!             })
//!             .on_enable(|service: ResMut<MyService>| -> Result<(), MyError> {
//!                 // Enabling and disabling services just require an empty return
//!                 // value.
//!             })
//!             .on_fail(
//!                 |error: In<ServiceErrorKind<MyError>>,
//!                  _some_sys_params: AssetServer| {
//!                     // Failure takes in an error value. There are a few kinds.
//!                     // This hook will fire if any error is encountered.
//!                     // In some cases, you will receive a warning. This hook will
//!                     // fire then, too.
//!                 },
//!             ),
//!             // I'll leave the rest for you to discover.
//!     );
//!
//!     // From here on out you make your app like normal, creating and reacting to
//!     // service changes like any other event.
//!     app.add_observer(|trigger: Trigger<MyServiceInitialized>| { /* ... */ });
//! }
//!
//! ## Tips for library authors
//!
//! If you want to create a service crate, you can create an extension trait for your service type.
//! ```rust, skip
//! pub trait ExposeMySpec {
//!     pub fn spec() -> MyServiceSpec;
//! }
//! ```

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
