#![allow(unused, reason = "it's just an example")]
use bevy::prelude::*;
use bevy_ecs::world::CommandQueue;
use q_service::prelude::*;

mod common;
use common::*;

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
    fn build(scope: &mut ServiceScope<MyService>) {
        // Depend on other services...
        scope
            .add_dep::<MyOtherService>()
            // resources...
            .add_resource::<MyResource>()
            // ... or assets.
            .add_asset::<MyAsset>("my/asset/path.toml");

        // Add systems with all the flexibility you're used to.
        scope.add_systems(
            Update,
            (sys_a, sys_b)
                .chain()
                // Services have their own run conditions.
                .run_if(service_up::<SomeOtherService>()),
        );

        // Define service behavior with possibly-asychrnonous
        // (de)initialization hooks.
        scope.init_with(my_init).deinit_with(my_deinit);

        // React to state changes immediately with service hooks.
        scope.on_up(my_up).on_down(my_down);

        // Automatically run this system on startup.
        scope.is_startup(true);
    }
}

// You can asychronously (de)initialize your service by returning
// an AsyncHook task to poll.
fn my_init() -> InitResult {
    let hook = AsyncHook::io_task(async |_q: CommandQueue| Ok(()));
    Ok(Some(hook))
}
// ... or keep it synchronous by returning Ok(None)
fn my_deinit() -> DeinitResult {
    Ok(None)
}

// Hooks can be any Bevy system.
fn my_up(mut commands: Commands) -> UpResult {
    commands.trigger(SomeEvent);
    Ok(())
}
// ... but they must match the hook signature.
fn my_down(reason: In<DownReason>) {
    // do something
}

fn observe_status_update(
    // You can react to changes using Observers or EventReaders.
    // EventReaders are preferred.
    mut reader: EventReader<EnterServiceState<MyService>>,
    // You can access services using SystemParams.
    other_service: ServiceRef<MyOtherService>,
    mut commands: Commands,
) {
    // You can update services using Commands.
    commands.spin_service_down::<SomeOtherService>();

    // Don't worry, there are aliases for these.
    for event in reader.read() {
        match &**event {
            ServiceStatus::Down(DownReason::Failed(e)) => todo!(),
            ServiceStatus::Down(DownReason::SpunDown) => todo!(),
            ServiceStatus::Down(DownReason::Uninitialized) => todo!(),
            ServiceStatus::Deinit(down_reason) => todo!(),
            ServiceStatus::Init => todo!(),
            ServiceStatus::Up => todo!(),
        }
    }
}

#[derive(Default, Debug)]
struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        // Add your service to a plugin,
        // or directly on the application.
        app.register_service::<MyService>();
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(MyPlugin);
    app.run();
}
