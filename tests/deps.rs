mod common;
use bevy::prelude::*;
use common::*;
use q_service::prelude::*;

#[test]
#[should_panic]
fn deps_fail_on_cycle() {
    let mut app = setup();
    app.add_service(
        TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle()]),
    )
    .add_service(
        TestService2::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService::handle()]),
    );
    app.update();
}

#[test]
fn dependency_initialization() {
    let mut app = setup();
    app.add_service(
        TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle()]),
    );
    app.add_service(TestService2::default_spec().with_deps(vec![TestService3::handle()]));
    app.add_service(TestService3::default_spec());

    // TODO: Should un-added services get automatically picked up and
    // initialized?

    app.update();
    let state = &app.world().resource::<TestService>().state;
    assert_eq!(state, &ServiceState::Enabled);
    let state = &app.world().resource::<TestService2>().state;
    assert_eq!(state, &ServiceState::Enabled);
    let state = &app.world().resource::<TestService3>().state;
    assert_eq!(state, &ServiceState::Enabled);
}

#[test]
fn failure_propogation() {
    let mut app = setup();
    app.add_service(
        TestService::default_spec()
            .is_startup(true)
            .with_deps(vec![TestService2::handle()]),
    );
    app.add_service(TestService2::default_spec().with_deps(vec![TestService3::handle()]));
    app.add_service(TestService3::default_spec().on_init(|| Err(TestErr::A)));
    app.update();
    let err_str = TestErr::A.to_string();
    app.world_mut()
        .resource_scope(|_world, s: Mut<TestService>| {
            let state = &s.state;
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
            let state = &s.state;
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
            let state = &s.state;
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
