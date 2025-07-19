# q_service

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)
[![Crates.io](https://img.shields.io/crates/v/q_service.svg)](https://crates.io/crates/q_service)
[![Downloads](https://img.shields.io/crates/d/q_service.svg)](https://crates.io/crates/q_service)
[![Docs](https://docs.rs/q_service/badge.svg)](https://docs.rs/q_service/latest/q_service/)
[![CI](https://github.com/ada-x64/q_service/actions/workflows/ci.yaml/badge.svg)](https://github.com/ada-x64/q_service/actions)
[![codecov](https://codecov.io/github/ada-x64/q_service/graph/badge.svg?token=2gqZobeujo)](https://codecov.io/github/ada-x64/q_service)
[![enbyware](https://pride-badges.pony.workers.dev/static/v1?label=enbyware&labelColor=%23555&stripeWidth=8&stripeColors=FCF434%2CFFFFFF%2C9C59D1%2C2C2C2C "they/she")](https://en.pronouns.page/are/they&she)

This crate aims to bring the service model to Bevy.

Bevy's ECS is like an operating system's process scheduler. It takes systems
(processes) that operate on data (files) and schedules them appropriately.
Well, if we have an OS analogue, we need an analogue for services.

This crate is loosely modelled after [systemd](https://systemd.io).
It doesn't manage PID1, but it does manage user services, including those
necessary for startup.

Services are resources which have built-in state, associated data, and
managed dependencies. This crate extends the Bevy ECS to include this
functionality. Because it is an engine extension, _there is no associated
plugin._ Simply import the prelude and you're good to go.

I have tried to make this documentation intuitive to explore in your IDE,
assuming you're using rust-analyzer or a similar LSP. You might get more out
of reading it as you use it rather than reading it all ahead of time.

## Features

### Dependency management

Dependency management in Bevy can be annoying, whether it's busy-waiting for asset loads or manually
checking that dependent plugins and actions have been performed. This crate aims to fix that.
Services allow you to declare your dependencies ahead of time so that they're ready to roll
whenever you are.

Dependencies can be assets, resource, or other services.

__Note:__ Currently depedency management happens synchrnonously, so we're only halfway to meeting this crate's stated purpose. Once asynchronous initialization is finished, we'll be able to handle Assets.

### Lifecycle events

In order to track dependencies you need to know if the service is currently initialized, whether it's failed, if it's enabled or disabled. In addition, services have data types associated with them so that you can update and react to these data changes. So, services provide you a way to generate reactive resources for your systems.

## Example usage
```rust
use bevy::prelude::*;
use q_service::prelude::*;
use thiserror::Error;

// First, you need to define your service variables.
// Services are uniquely determined by three types:
// ServiceLabel, ServiceError, and ServiceData.
// ServiceLabel will be defined for you when you declare
// the service with `service!`, but you'll have to manually
// define your data and error types.

#[derive(ServiceError, Error, Debug, Clone, PartialEq)]
pub enum MyError {}

// If your service doesn't need any data, you can just pass in ().
#[derive(ServiceData, Debug, Clone, PartialEq, Default)]
pub struct MyData {}

// Declare the service!
// This will create a bunch of useful aliases and the
// ServiceLabel type.
service!(MyService, MyData, MyError);
// Services can share data and error types so long as their names are distinct.
service!(MyOtherService, MyData, MyError);

// Next, add the service to the application.
fn main() {
    let mut app = App::new();
    // We use a ServiceSpec to declaratively define the behavior of the service.
    app
#       .add_plugins(DefaultPlugins)
        .add_service(
        MyService::default_spec()
            // This service will initialize in the Startup schedule.
            // By default, services are lazily initialized whenever they are
            // enabled.
            .is_startup(true)
            // This service has some dependencies!
            // Before it initializes, it will initialize all of its
            // dependencies, and their dependencies.
            .with_deps(vec![
                // Service handles provide a convenient way to refer to
                // services without passing them around.
                // They're zero-sized types which act as a shorthand for the
                // service's type specification.
                MyOtherService::handle().into(),
            ])
            // Services can hold arbitrary data types. This can be modified by using
            // lifecycle hooks or commands. You can listen for changes by using events.
            .with_data(MyData {
                /* ... */
            })
            // The service's behavior is largely defined by its lifecycle hooks.
            // There are five main lifecycle events.
            // The first is initialization.
            .on_init(|world: &mut World| -> Result<bool, MyError> {
                // This can be any system function, exclusive or otherwise.
                // It just has to have the right return variable.
                // Initialization can proceed to enable or disable the
                // service, depending on the return value.
                Ok(true)
            })
            // Next are enabling and disabling.
            .on_enable(|service: ResMut<MyService>| -> Result<(), MyError> {
                // Enabling and disabling services just require an empty return
                // value.
                Ok(())
            })
            // Then there's data transformation.
            .on_update(|data: In<MyData>| -> Result<MyData, MyError> {
                Ok(data.clone())
            })
            // ... and finally, failure handling.
            // In some cases, you will receive a warning. This hook will
            // fire then, too. Warnings don't change the service's state.
            .on_failure(
                |error: In<ServiceErrorKind<MyError>>| {
                    // ...
                },
            ),
    );

    // From here on out you make your app like normal, creating and reacting to
    // service changes like any other event.
    app.add_observer(|trigger: Trigger<MyServiceInitialized>| { /* ... */ });
}
```

## Tips for library authors

If you want to create a service crate, you can create an extension trait for your service type.
```rust
# use q_service::prelude::*;
# use bevy::prelude::*;
# #[derive(ServiceError, thiserror::Error, Debug, Clone, PartialEq)]
# pub enum MyError {}
# service!(MyService, (), MyError);
// lib.rs
pub trait ExposeMySpec {
    fn spec() -> MyServiceSpec;
}
impl ExposeMySpec for MyService {
    fn spec() -> MyServiceSpec {
        MyService::default_spec()
            //...
    }
}

// main.rs
pub fn main() {
    App::new().add_service(MyService::spec());
}
```

... or just export a plugin:
```rust
# use q_service::prelude::*;
# use bevy::prelude::*;
# #[derive(ServiceError, thiserror::Error, Debug, Clone, PartialEq)]
# pub enum MyError {}
# service!(MyService, (), MyError);
# pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_service(
            MyService::default_spec()
            // ...
        );
    }
}
# pub fn main() {}
```

## Bevy tracking

| q_service | bevy |
| --- | --- |
| 0.1 | 0.16 |
