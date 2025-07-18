mod common;
use bevy::prelude::*;
use common::*;
use q_service::{deps::DepInitErr, prelude::*};

#[test]
fn deps_fail_on_cycle() {
    let res = std::panic::catch_unwind(|| {
        let mut app = setup();
        let ts1 = TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle().into()]);
        let ts2 = TestService2::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService::handle().into()]);
        app.add_service(ts1).add_service(ts2);
    });
    let expected = "DepCycle";
    let err = res
        .unwrap_err()
        .downcast::<String>()
        .expect("Wrong downcast.");
    assert!(err.contains(expected))
}

#[test]
fn deps_fail_on_loop() {
    let res = std::panic::catch_unwind(|| {
        let mut app = setup();
        app.add_service(
            TestService::default_spec()
                .is_startup(true)
                .with_deps(vec![TestService::handle().into()]),
        );
    });
    let expected = "DepLoop";
    let err = res
        .unwrap_err()
        .downcast::<String>()
        .expect("Wrong downcast.");
    assert!(err.contains(expected))
}

#[test]
fn dependency_initialization() {
    let mut app = setup();
    app.add_service(
        TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle().into()]),
    );
    app.add_service(TestService2::default_spec().with_deps(vec![TestService3::handle().into()]));
    app.add_service(TestService3::default_spec());

    app.update();
    let state = app.world().resource::<TestService>().state();
    assert!(matches!(state, ServiceState::Enabled));
    let state = app.world().resource::<TestService2>().state();
    assert!(matches!(state, ServiceState::Enabled));
    let state = app.world().resource::<TestService3>().state();
    assert!(matches!(state, ServiceState::Enabled));
}

#[test]
fn failure_propogation() {
    let mut app = setup();
    app.add_service(
        TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle().into()]),
    );
    app.add_service(TestService2::default_spec().with_deps(vec![TestService3::handle().into()]));
    app.add_service(TestService3::default_spec().on_init(|| Err(TestErr::A)));
    app.update();
    let err_str = TestErr::A.to_string();
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService>| {
            let state = s.state();
            debug!("Checking state {state:#?}");
            match state {
                ServiceState::Failed(ServiceErrorKind::Dependency(a, b, e)) => {
                    assert_eq!(a, &TestService::handle().to_string());
                    assert_eq!(b, &TestService2::handle().to_string());
                    assert!(e.contains(&err_str));
                }
                _ => {
                    panic!()
                }
            }
        });
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService2>| {
            let state = s.state();
            match state {
                ServiceState::Failed(ServiceErrorKind::Dependency(a, b, e)) => {
                    assert_eq!(a, &TestService2::handle().to_string());
                    assert_eq!(b, &TestService3::handle().to_string());
                    assert!(e.contains(&err_str));
                }
                _ => {
                    panic!()
                }
            }
        });
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService3>| {
            let state = s.state();
            match state {
                ServiceState::Failed(ServiceErrorKind::Own(e)) => {
                    assert!(matches!(e, TestErr::A));
                }
                _ => {
                    panic!()
                }
            }
        });
}
