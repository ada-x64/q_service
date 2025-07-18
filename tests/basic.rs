use bevy::prelude::*;
use q_service::prelude::*;
mod common;
use common::*;

#[test]
fn simple() {
    let mut app = setup();
    app.add_service(TestService::default_spec());
    app.update();
    let world = app.world_mut();
    let s = world.resource_mut::<TestService>();
    assert!(matches!(s.state(), ServiceState::Uninitialized));
}

#[test]
fn hook_failure() {
    let mut app = setup();
    app.add_service(TestService::default_spec().is_startup(true).on_init(|| {
        info!("In hook");
        Err(TestErr::A)
    }));
    app.update();
    let world = app.world_mut();
    let service = world.resource_mut::<TestService>();
    assert_eq!(
        service.state(),
        &ServiceState::Failed(ServiceErrorKind::Own(TestErr::A))
    );
}

#[test]
fn manual_init() {
    let mut app = setup();
    app.add_service(TestService::default_spec());
    app.update();
    app.world_mut()
        .commands()
        .init_service(TestService::handle());
    app.update();
    let world = app.world_mut();
    let service = world.resource_mut::<TestService>();
    assert_eq!(service.state(), &ServiceState::Enabled);
}

#[derive(Resource, Debug, Default, PartialEq)]
pub struct TestHooks {
    init: bool,
    enable: bool,
    disable: bool,
    fail: bool,
    update: bool,
}

#[test]
fn hooks() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    let spec = TestService::default_spec()
        .is_startup(true)
        .on_init(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("init");
            hooks_ran.init = true;
            Ok(true)
        })
        .on_enable(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("enable");
            hooks_ran.enable = true;
            Ok(())
        })
        .on_update(|_: In<()>, mut hooks_ran: ResMut<TestHooks>| {
            debug!("update");
            hooks_ran.update = true;
            Ok(())
        })
        .on_disable(|mut hooks_ran: ResMut<TestHooks>| {
            debug!("disable");
            hooks_ran.disable = true;
            Err(TestErr::A)
        })
        .on_failure(
            |_err: In<ServiceErrorKind<TestErr>>, mut hooks_ran: ResMut<TestHooks>| {
                debug!("failure");
                hooks_ran.fail = true;
            },
        );
    println!("{spec:#?}");
    app.add_service(spec);
    app.update();
    app.world_mut()
        .commands()
        .update_service(TestService::handle(), ());
    app.update();
    app.world_mut()
        .commands()
        .disable_service(TestService::handle());
    app.update();
    assert_eq!(
        app.world_mut().resource::<TestHooks>(),
        &TestHooks {
            init: true,
            enable: true,
            disable: true,
            fail: true,
            update: true,
        }
    );
}

#[test]
fn events() {
    let mut app = setup();
    app.init_resource::<TestHooks>();
    // NOTE: This fails with `is_startup(true)`. Probably because observers need
    // to be instantiated before events can fire.
    app.add_service(TestService::default_spec())
        .add_observer(
            |t: Trigger<TestServiceStateChange>,
             mut r: ResMut<TestHooks>,
             mut commands: Commands| {
                match t.event().0.1 {
                    ServiceState::Initializing => {
                        r.init = true;
                    }
                    ServiceState::Enabled => {
                        r.enable = true;
                        commands.update_service(TestService::handle(), ())
                    }
                    ServiceState::Disabled => {
                        r.disable = true;
                        commands
                            .fail_service(TestService::handle(), ServiceErrorKind::Own(TestErr::A));
                    }
                    ServiceState::Failed(_) => r.fail = true,
                    _ => {}
                }
            },
        )
        .add_observer(
            |_: Trigger<TestServiceUpdated>, mut r: ResMut<TestHooks>, mut commands: Commands| {
                r.update = true;
                commands.disable_service(TestService::handle());
            },
        );
    app.world_mut()
        .commands()
        .init_service(TestService::handle());
    app.update();
    app.update();
    assert_eq!(
        app.world_mut().resource::<TestHooks>(),
        &TestHooks {
            init: true,
            enable: true,
            disable: true,
            fail: true,
            update: true,
        }
    );
}

#[derive(Resource, Default, Debug, PartialEq)]
struct Ran {
    service_has_state: bool,
    service_uninitialized: bool,
    // service_initializing: bool,
    service_enabled: bool,
    service_disabled: bool,
    service_failed: bool,
    service_failed_with_error: bool,
}

macro_rules! check_run_condition {
    ($app:ident, $condition:ident) => {
        $app.add_systems(
            Update,
            (|mut ran: ResMut<Ran>| {
                ran.$condition = true;
            })
            .run_if($condition(TestService::handle())),
        );
    };
}

#[test]
fn run_conditions() {
    let mut app = setup();
    app.init_resource::<Ran>();
    app.add_service(TestService::default_spec());
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_has_state = true;
        })
        .run_if(service_has_state(
            TestService::handle(),
            ServiceState::Enabled,
        )),
    );
    app.add_systems(
        Update,
        (|mut ran: ResMut<Ran>| {
            ran.service_failed_with_error = true;
        })
        .run_if(service_failed_with_error(
            TestService::handle(),
            ServiceErrorKind::Own(TestErr::A),
        )),
    );
    check_run_condition!(app, service_uninitialized);
    // check_run_condition!(app, service_initializing);
    check_run_condition!(app, service_enabled);
    check_run_condition!(app, service_disabled);
    check_run_condition!(app, service_failed);

    app.update();
    app.world_mut()
        .commands()
        .init_service(TestService::handle());
    app.update();
    app.world_mut()
        .commands()
        .enable_service(TestService::handle());
    app.update();
    app.world_mut()
        .commands()
        .disable_service(TestService::handle());
    app.update();
    app.world_mut()
        .commands()
        .fail_service(TestService::handle(), ServiceErrorKind::Own(TestErr::A));
    app.update();

    let all_ok = Ran {
        service_has_state: true,
        service_uninitialized: true,
        // TODO: This will only be called if initializing takes more than one
        // frame! Need async init before we can test this.
        // service_initializing: true,
        service_enabled: true,
        service_disabled: true,
        service_failed: true,
        service_failed_with_error: true,
    };
    assert_eq!(app.world().resource::<Ran>(), &all_ok);
}

#[derive(Resource, Default, Debug, PartialEq)]
struct Errors(Vec<ServiceErrorKind<TestErr>>);

#[test]
fn uninitialized() {
    let mut app = setup();
    app.add_service(TestService::default_spec().on_failure(
        |e: In<_>, mut errs: ResMut<Errors>| {
            errs.0.push(e.0);
        },
    ))
    .init_resource::<Errors>()
    .add_systems(Startup, |mut commands: Commands| {
        commands.disable_service(TestService::handle()); // this should fail.
        commands.enable_service(TestService::handle()); // this should initialize.
        commands.init_service(TestService::handle()); // this should fail.
    });
    app.update();
    app.update();
    let errors = app.world().resource::<Errors>();
    let display_name = TestService::handle().to_string();
    assert_eq!(
        errors.0,
        vec![
            ServiceErrorKind::Uninitialized(display_name.clone()),
            ServiceErrorKind::AlreadyInitialized(display_name.clone()),
        ]
    );
    let state = &app.world().resource::<TestService>().state();
    assert!(matches!(state, ServiceState::Enabled));
}

// TODO: Async initialization
// ------> maybe do Initializing(f32) (gloss as percentage)
