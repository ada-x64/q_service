This module is about the service lifecycle.

Services go through a few distinct phases, as represented by the chart
below. ![service-lifecycle.png](TODO)

## Hooks

When initializing a service, you can add [hooks](./hooks) to each lifecycle
phase. For example:

```rust
# use bevy::prelude::*;
# use q_service::prelude::*;
#
# #[derive(ServiceError, thiserror::Error, Debug, PartialEq, Clone)]
# pub enum ExampleError {}
#
# service!(ExampleService, (), ExampleError);
#
# fn main() {
#     let mut app = App::new();
    app.add_service(
        ExampleService::default_spec()
            // This can be any system, as long as the return type is correct.
            .on_init(|| {
                Ok(true)
            })
    );
# }
```

Hooks can take in any Bevy function system. Each hook has its own required signature.

| Hook | Signature |
| --- | --- |
| Init | `(<system_params>) -> Result<bool, E>` |
| Enable, Disable | `(<system_params>) -> Result<(), E>` |
| Update | `(data: In<D>, <system_params>) -> Result<D, E>` |
| Fail | `(error: In<SystemErrorKind<E>>, <system_params>) -> ()` |

... where `T, D, E` are your service's [ServiceLabel](crate::data::ServiceLabel), [ServiceData](crate::data::ServiceData), and [ServiceError](crate::data::ServiceError) types, respectiely.

## Commands

Commands can be used to call service lifecycle events directly. If you need something to execute _now_, this is useful. Note that this uses [World::resource_scope](bevy_ecs::prelude::World::resource_scope) internally, so the service is temporarily taken out of the World while the command executes.

```rust
# use q_service::prelude::*;
# use bevy::prelude::*;
#
# #[derive(ServiceError, thiserror::Error, PartialEq, Debug, Clone)]
# pub enum MyError {}
# #[derive(ServiceData, PartialEq, Default, Debug, Clone)]
# pub struct MyData;
# service!(MyService, MyData, MyError);
# pub fn main() {
# let mut app = App::new();
# let mut world = app.world_mut();
# let mut commands = world.commands();
commands.update_service(MyService::handle(), MyData);
# }
```


## Events

You can react to service state changes using events.
Events fire _after_ the service has already updated.
If you want to modify the behavior of your service, you'll need to use hooks.

```rust
# use q_service::prelude::*;
# use bevy::prelude::*;
#
# #[derive(ServiceError, thiserror::Error, PartialEq, Debug, Clone)]
# pub enum MyError {}
# service!(MyService, (), MyError);
# pub fn main() {
# let mut app = App::new();
app.add_observer(|trigger: Trigger<MyServiceEnabled>| {/*...*/});
# }
```

## Run Conditions

This crate defines a few run conditions for services, in case you want to do something like the below:

```rust
# use q_service::prelude::*;
# use bevy::prelude::*;
#
# #[derive(ServiceError, thiserror::Error, PartialEq, Debug, Clone)]
# pub enum MyError {}
# service!(MyService, (), MyError);
# pub fn my_systems() {}
# pub fn main() {
# let mut app = App::new();
app.add_systems(Update, (my_systems).run_if(service_enabled(MyService::handle())));
# }
```
