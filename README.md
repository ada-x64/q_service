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
It doesn't manage PID1 (startup), but it does manage user services, including
those necessary for startup.

Services are resources which have built-in state, scoped systems, and
managed dependencies. This crate extends the Bevy ECS to include this
functionality. Because it is an engine extension, _there is no associated
plugin._ Simply import the prelude and you're good to go.

## Features

### System scoping

A service is designed to manage processes. Accordingly, services are designed to
manage systems. All this does is add systems to a SystemSet which only runs
when the service is up. Simple, but encapsulating!

See [ServiceScope](crate::prelude::ServiceScope) for more info.

### Dependency management

Dependency management in Bevy can be annoying, whether it's busy-waiting for
asset loads or manually checking that dependent plugins have been loaded and
actions have been performed. This crate aims to fix that. Services allow you to
declare your dependencies ahead of time so that they're ready to roll whenever
you are.

Dependencies can be assets, resources, or other services. Adding them to your
service is as easy as stating `service_scope.add_asset::<MyAsset>(asset_path)`.

See [GraphData](crate::prelude::GraphData) for more info.

### State Management and Lifecycle Events

In order to track dependencies you need to know if the associated service is currently up, down, or in a loading state. So, I've implemented some basic state management mechanics, including the ability to hook into state changes directly on the service or by watching for events.

See [Lifecycle](crate::lifecycle) for more info.

## Example usage

```rust
use bevy::prelude::*;
use q_service::prelude::*;

# #[derive(Resource, Debug, Default)]
# pub struct MyOtherService;
# impl Service for MyOtherService {
#    fn build(_) {}
# }
# #[derive(Resource)]
# pub struct MyResource;
# #[derive(Asset)]
# pub struct MyAsset;

// Declare your service like this.
// Services must implement these three traits.
#[derive(Resource, Debug, Default)]
pub struct MyService {
    // you can have data here if you want.
}
impl Service for MyService {
    // This function will build the service at registration.
    // This is where you define the service's behavior --
    // its dependencies, systems, and lifecycle hooks.
    fn build(scope: &mut ServiceScope) {
        // Depend on other services...
        scope.add_dep::<MyOtherService>()
            // resources...
            .add_resource::<MyResource>()
            // ... or assets.
            .add_asset::<MyAsset>("my/asset/path.toml");

        // Add systems with all the flexibility you're used to.
        scope.add_systems(
            Update,
            (sys_a, sys_b)
                .chain()
                .run_if(my_condition)
        );

        // Define service behavior with possibly-asychrnonous
        // (de)initialization hooks.
        scope.init_with(my_init)
            .deinit_with(my_deinit);

        // React to state changes immediately with service hooks.
        scope.on_up(my_up)
            .on_down(my_down);
    }
}

// You can react to changes using observers...
fn observe_status_update(trigger: Trigger<EnterServiceState<MyService>>>) {
    // ...
}

// ... or EventReaders
fn read_state_change(reader: EventReader<ServiceStateChange<MyService>>) {
    // ...
}

#[derive(Default, Debug)]
struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(app: &mut App) {
        // Add your service to a plugin,
        // or directly on the application.
        app.add_service::<MyService>();
    }
}

```

For more complete examples, see the [github repo](https://github.com/ada-x64/q_service)

## Bevy tracking

| q_service | bevy |
| --------- | ---- |
| 0.1, 0.2  | 0.16 |
