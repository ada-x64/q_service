use std::time::{Duration, Instant};

use bevy::prelude::*;
use q_service::prelude::*;
mod common;
use common::*;

#[derive(Resource, Default, Debug)]
struct Simple;
impl Service for Simple {
    fn build(_: &mut ServiceScope<Self>) {}
}

#[test]
fn simple() {
    let mut app = setup();
    app.register_service::<Simple>();
    app.update();
    let status = app.world().service::<Simple>().status();
    assert!(matches!(
        status,
        ServiceStatus::Down(DownReason::Uninitialized)
    ));

    app.world_mut().commands().spin_service_up::<Simple>();
    app.update();
    let status = app.world().service::<Simple>().status();
    assert!(matches!(status, ServiceStatus::Up));

    app.world_mut().commands().spin_service_down::<Simple>();
    app.update();
    let status = app.world().service::<Simple>().status();
    assert!(matches!(status, ServiceStatus::Down(DownReason::SpunDown)));
}

#[derive(Resource, Default, Debug)]
struct NoDupes;
impl Service for NoDupes {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.on_up(count_up).is_startup(true);
    }
}

#[test]
fn no_dupes() {
    let mut app = setup();
    app.init_resource::<Count>()
        .register_service::<NoDupes>()
        .register_service::<NoDupes>();
    app.update();
    let count = app.world().resource::<Count>();
    assert_eq!(count.up, 1);
}

#[derive(Resource, Default, Debug)]
struct HookFailure;
impl Service for HookFailure {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.init_with(|| Err("oh no".into())).is_startup(true);
    }
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.register_service::<HookFailure>();
    app.update();
    let status = app.world().service::<HookFailure>().status();
    matches!(
        status,
        ServiceStatus::Down(DownReason::Failed(ServiceError::Own(_)))
    );
}

#[derive(Resource, Default, Debug)]
struct Hooks;
impl Service for Hooks {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .init_with(count_init)
            .deinit_with(count_deinit)
            .on_up(count_up)
            .on_down(count_down);
    }
}

#[test]
fn hooks() {
    let mut app = setup();
    app.init_resource::<Count>().register_service::<Hooks>();
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.update();
    assert_eq!(
        app.world_mut().resource::<Count>(),
        &Count {
            init: 1,
            up: 1,
            down: 1,
            deinit: 1,
        }
    );
}

#[derive(Default, Resource, Debug)]
struct Events;
impl Service for Events {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.init_with(noop_init).deinit_with(noop_init);
    }
}
fn noop_init() -> InitResult {
    let hook = AsyncHook::io_task(async |_| Ok(()));
    Ok(Some(hook))
}

#[test]
fn respond_to_events() {
    let mut app = setup();
    app.init_resource::<Count>();
    app.register_service::<Events>().add_systems(
        Update,
        |mut events: EventReader<EnterServiceState<Events>>,
         mut r: ResMut<Count>,
         mut commands: Commands| {
            for event in events.read() {
                match &**event {
                    ServiceStatus::Init => r.init += 1,
                    ServiceStatus::Up => {
                        r.up += 1;
                        commands.spin_service_down::<Events>();
                    }
                    ServiceStatus::Deinit(_) => r.deinit += 1,
                    ServiceStatus::Down(_) => {
                        r.down += 1;
                    }
                }
            }
        },
    );
    app.world_mut().commands().spin_service_up::<Events>();
    app.update(); // init
    app.update(); // up
    app.update(); // deinit
    app.update(); // down
    assert_eq!(
        app.world_mut().resource::<Count>(),
        &Count {
            init: 1,
            up: 1,
            down: 1,
            deinit: 1,
        }
    );
}

// #[test]
// fn observers() {
//     let mut app = setup();
//     app.init_resource::<Count>();
//     app.register_service::<Events>().add_observer(
//         |trigger: Trigger<EnterServiceState<Events>>,
//          mut r: ResMut<Count>,
//          mut commands: Commands| {
//             match &**trigger.event() {
//                 ServiceStatus::Init => {
//                     debug!("init!");
//                     r.init += 1;
//                 }
//                 ServiceStatus::Up => {
//                     debug!("up!");
//                     r.up += 1;
//                     commands.spin_service_down::<Events>();
//                 }
//                 ServiceStatus::Deinit(_) => {
//                     debug!("deinit!");
//                     r.deinit += 1;
//                 }
//                 ServiceStatus::Down(_) => {
//                     debug!("down!");
//                     r.down += 1;
//                 }
//             }
//         },
//     );
//     // note: These are all observers so they _should_ be happening ASAP,
//     // but async polling is happening once per update, making this relatively slow...
//     app.world_mut().commands().spin_service_up::<Events>();
//     app.update(); // init
//     app.update(); // up
//     app.update(); // deinit
//     app.update(); // down
//     assert_eq!(
//         app.world_mut().resource::<Count>(),
//         &Count {
//             init: 1,
//             up: 1,
//             down: 1,
//             deinit: 1,
//         }
//     );
// }

#[derive(Resource, Default, Debug, PartialEq)]
struct Ran {
    service_has_state: bool,
    service_initializing: bool,
    service_up: bool,
    service_deinitializing: bool,
    service_down: bool,
    service_failed: bool,
    service_failed_with_error: bool,
}

macro_rules! check_run_condition {
    ($app:ident, $t:ty, $condition:ident) => {
        $app.add_systems(
            Update,
            (|mut ran: ResMut<Ran>| {
                ran.$condition = true;
            })
            .run_if($condition::<$t>()),
        );
    };
}

#[derive(Default, Resource, Debug)]
struct RunConditions;
impl Service for RunConditions {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .init_with(run_condition_async)
            .deinit_with(run_condition_async);
    }
}

fn busy_wait(millis: u64) {
    let start = Instant::now();
    while Instant::now().duration_since(start) <= Duration::from_millis(millis) {}
}
fn run_condition_async() -> InitResult {
    let task = AsyncHook::async_compute_task(async |_| {
        debug!("In AsyncComputeTaskPool");
        busy_wait(100);
        debug!("...AsyncComputeTaskPool DONE");
        Ok(())
    });
    Ok(Some(task))
}

#[test]
fn run_conditions() {
    let mut app = setup();
    app.init_resource::<Ran>();
    app.register_service::<RunConditions>();
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_has_state = true;
        })
        .run_if(service_has_status::<RunConditions>(ServiceStatus::Up)),
    );
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_failed_with_error = true;
        })
        .run_if(service_failed_with_error::<RunConditions>(
            ServiceError::Own("oh no".into()),
        )),
    );
    check_run_condition!(app, RunConditions, service_initializing);
    check_run_condition!(app, RunConditions, service_up);
    check_run_condition!(app, RunConditions, service_down);
    check_run_condition!(app, RunConditions, service_deinitializing);
    check_run_condition!(app, RunConditions, service_failed);

    app.update(); // service_down
    app.world_mut()
        .commands()
        .spin_service_up::<RunConditions>();
    app.update(); // service_initializing
    busy_wait(100); // wait for it to be finished...
    app.update(); // service_up, service_has_status(up)
    app.world_mut()
        .commands()
        .fail_service::<RunConditions>(ServiceError::Own("oh no".into()));
    app.update(); // deinit
    busy_wait(100); // wait for it to be finished...
    app.update(); // service_down, service_failed, service_failed_with

    let all_ok = Ran {
        service_has_state: true,
        service_initializing: true,
        service_up: true,
        service_deinitializing: true,
        service_down: true,
        service_failed: true,
        service_failed_with_error: true,
    };
    assert_eq!(app.world().resource::<Ran>(), &all_ok);
}

#[test]
fn redundant_calls() {
    let mut app = setup();
    app.register_service::<Hooks>().init_resource::<Count>();
    // this should warn and do nothing.
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.update();
    // this should initialize.
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    // this should  warn and do nothing.
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    // this should re-initialize
    app.world_mut().commands().restart_service::<Hooks>();
    app.update();
    let count = app.world().resource::<Count>();
    assert_eq!(
        count,
        &Count {
            up: 2,
            init: 2,
            down: 0,
            deinit: 0,
        }
    );
    let status = app.world().service::<Hooks>().status();
    assert!(matches!(status, ServiceStatus::Up));
}

#[test]
fn command_priority() {
    let mut app = setup();
    app.register_service::<Hooks>().init_resource::<Count>();
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    assert!(app.world_mut().service::<Hooks>().status().is_up());
    // this should spin down.
    // app is up so priority of down = 2, priority of up = 3
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    assert!(app.world_mut().service::<Hooks>().status().is_down());
    // this should spin up.
    // app is down so priority of down = 3, priority of up = 2
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.update();
    assert!(app.world_mut().service::<Hooks>().status().is_up());
    // this should spin up.
    // app is up so priority of down = 2, but priority of restart = 1
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.world_mut().commands().restart_service::<Hooks>();
    app.update();
    assert!(app.world_mut().service::<Hooks>().status().is_up());
    // this should fail.
    // priority of fail = 0, will overpower everything
    app.world_mut().commands().spin_service_down::<Hooks>();
    app.world_mut().commands().spin_service_up::<Hooks>();
    app.world_mut().commands().restart_service::<Hooks>();
    app.world_mut()
        .commands()
        .fail_service::<Hooks>(ServiceError::Own("oh no".to_string()));
    app.update();
    assert!(app.world_mut().service::<Hooks>().status().is_failed());
}
