mod common;
use bevy::prelude::*;
use common::*;
use q_service::prelude::*;

#[derive(Resource, Debug, Default)]
struct Cycle1;
impl Service for Cycle1 {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Cycle2>().is_startup(true);
    }
}
#[derive(Resource, Debug, Default)]
struct Cycle2;
impl Service for Cycle2 {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Cycle1>().is_startup(true);
    }
}

#[test]
fn deps_fail_on_cycle() {
    let res = std::panic::catch_unwind(|| {
        let mut app = setup();
        app.register_service::<Cycle1>()
            .register_service::<Cycle2>()
            .update()
    });
    let expected = "DepCycle";
    let err = res
        .unwrap_err()
        .downcast::<String>()
        .expect("Wrong downcast.");
    assert!(err.contains(expected))
}

#[derive(Resource, Debug, Default)]
struct Loop;
impl Service for Loop {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Loop>().is_startup(true);
    }
}
#[test]
#[should_panic]
fn deps_fail_on_loop() {
    let mut app = setup();
    app.register_service::<Loop>().update();
}

#[derive(Resource, Debug, Default)]
struct SimpleDepDep;
impl Service for SimpleDepDep {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<SimpleDep>().is_startup(true);
    }
}
#[derive(Resource, Debug, Default)]
struct SimpleDep;
impl Service for SimpleDep {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Simple>();
    }
}
#[derive(Resource, Debug, Default)]
struct Simple;
impl Service for Simple {
    fn build(_: &mut ServiceScope<Self>) {}
}

#[test]
fn deps_spin_up() {
    let mut app = setup();
    app.register_service::<SimpleDepDep>();
    app.register_service::<SimpleDep>();
    app.register_service::<Simple>();

    app.update();
    let world = app.world();
    status_matches!(world, SimpleDepDep, ServiceStatus::Up);
    status_matches!(world, SimpleDep, ServiceStatus::Up);
    status_matches!(world, Simple, ServiceStatus::Up);
}

#[derive(Resource, Debug, Default)]
struct DepDepFailure;
impl Service for DepDepFailure {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<DepFailure>().is_startup(true);
    }
}
#[derive(Resource, Debug, Default)]
struct DepFailure;
impl Service for DepFailure {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<FailOnInit>();
    }
}
#[derive(Resource, Debug, Default)]
struct FailOnInit;
impl Service for FailOnInit {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.init_with(|| Err("oh no".into()));
    }
}

#[test]
fn failure_propogation() {
    let mut app = setup();
    app.register_service::<DepDepFailure>();
    app.register_service::<DepFailure>();
    app.register_service::<FailOnInit>();

    app.update();
    let err_str = "oh no".to_string();
    let status = app.world().service::<DepDepFailure>().status();
    debug!(
        "Checking status for {} : {status:#?}",
        DepDepFailure::name()
    );
    match status {
        ServiceStatus::Down(DownReason::Failed(ServiceError::Dependency(ref dep, ref e))) => {
            assert_eq!(*dep, DepFailure::name());
            assert!(e.contains(&err_str));
        }
        _ => {
            panic!()
        }
    }
    debug!("Checking status for {} : {status:#?}", DepFailure::name());
    let status = app.world().service::<DepFailure>().status();
    match status {
        ServiceStatus::Down(DownReason::Failed(ServiceError::Dependency(ref dep, ref e))) => {
            assert_eq!(*dep, FailOnInit::name());
            assert!(e.contains(&err_str));
        }
        _ => {
            panic!()
        }
    }
    debug!("Checking status for {} : {status:#?}", FailOnInit::name());
    let status = app.world().service::<FailOnInit>().status();
    match status {
        ServiceStatus::Down(DownReason::Failed(ServiceError::Own(ref e))) => {
            assert_eq!(e.trim(), err_str);
        }
        _ => {
            panic!()
        }
    }
}

#[derive(Resource, Debug, Default)]
struct RedundantDep;
impl Service for RedundantDep {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .add_dep::<SimpleDep>()
            .add_dep::<Simple>()
            .is_startup(true);
    }
}

#[test]
fn redundant_deps() {
    let mut app = setup();
    app.register_service::<RedundantDep>();
    app.register_service::<SimpleDep>();
    app.register_service::<Simple>();
    app.update();
}

#[derive(Resource, Debug, Default)]
struct Path1_1;
impl Service for Path1_1 {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Path1_2>().is_startup(true);
    }
}
#[derive(Resource, Debug, Default)]
struct Path1_2;
impl Service for Path1_2 {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Simple>();
    }
}
#[derive(Resource, Debug, Default)]
struct Path2;
impl Service for Path2 {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.add_dep::<Simple>().is_startup(true);
    }
}

#[test]
fn multiple_supers() {
    let mut app = setup();
    app.register_service::<Path1_1>();
    app.register_service::<Path1_2>();
    app.register_service::<Path2>();
    app.register_service::<Simple>();
    app.update();
}

#[test]
fn deps_spin_down() {
    let mut app = setup();
    app.register_service::<SimpleDepDep>();
    app.register_service::<SimpleDep>();
    app.register_service::<Simple>();
    app.update();

    {
        let world = app.world_mut();
        status_matches!(world, SimpleDepDep, ServiceStatus::Up);
        status_matches!(world, SimpleDep, ServiceStatus::Up);
        status_matches!(world, Simple, ServiceStatus::Up);
        world.commands().spin_service_down::<SimpleDepDep>();
    }
    app.update();
    // this gets caught waiting for deps.
    // works fine if you update it again
    status_matches!(
        app.world(),
        SimpleDepDep,
        ServiceStatus::Down(DownReason::SpunDown)
    );
    status_matches!(
        app.world(),
        SimpleDep,
        ServiceStatus::Down(DownReason::SpunDown)
    );
    status_matches!(
        app.world(),
        Simple,
        ServiceStatus::Down(DownReason::SpunDown)
    );
}

#[test]
fn unregistered_dep() {
    let mut app = setup();
    app.register_service::<SimpleDep>();
    app.world_mut().commands().spin_service_up::<SimpleDep>();
    app.update();
    status_matches!(
        app.world(),
        SimpleDep,
        ServiceStatus::Down(DownReason::Failed(ServiceError::Dependency(..)))
    );
}

#[derive(Resource, Debug, Default, PartialEq)]
struct TestPassed(bool);

#[derive(Resource, Debug, Default)]
struct ResourceDep;
impl Service for ResourceDep {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .add_resource_with(|| TestPassed(true))
            .is_startup(true);
    }
}
#[test]
fn resource_dep() {
    let mut app = setup();
    app.register_service::<ResourceDep>();
    app.update();
    assert_eq!(
        app.world().get_resource::<TestPassed>(),
        Some(&TestPassed(true))
    );
    app.world_mut()
        .commands()
        .spin_service_down::<ResourceDep>();
    app.update();
    assert_eq!(app.world().get_resource::<TestPassed>(), None);
}
