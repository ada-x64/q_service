#![allow(unused, reason = "Actually used in macros")]

use bevy::log::LogPlugin;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use q_service::prelude::*;
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TestData;

#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub struct Count {
    pub up: u32,
    pub down: u32,
    pub init: u32,
    pub deinit: u32,
}
pub fn count_init(mut count: ResMut<Count>) -> InitResult {
    debug!("init");
    count.init += 1;
    Ok(None)
}
pub fn count_deinit(mut count: ResMut<Count>) -> DeinitResult {
    debug!("deinit");
    count.deinit += 1;
    Ok(None)
}
pub fn count_up(mut count: ResMut<Count>) -> UpResult {
    debug!("up");
    count.up += 1;
    Ok(())
}
pub fn count_down(_: In<DownReason>, mut count: ResMut<Count>) {
    debug!("down");
    count.down += 1;
}

pub fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        LogPlugin {
            filter: "trace".into(),
            ..Default::default()
        },
    ))
    .add_systems(Startup, || debug!("STARTUP"))
    .add_systems(Update, || debug!("UPDATE"));
    app
}

pub fn assert_status<T: Service>(world: &World, status: ServiceStatus) {
    let the_status = world.service::<T>().status();
    assert_eq!(the_status, status);
}

#[macro_export]
/// (world, type, status)
macro_rules! status_matches {
    ($world:expr, $t:ty, $status:pat) => {
        let the_status = $world.service::<$t>().status();
        assert!(matches!(the_status, $status));
    };
}

pub fn busy_wait(millis: u64) {
    let start = Instant::now();
    while Instant::now().duration_since(start) <= Duration::from_millis(millis) {}
}
