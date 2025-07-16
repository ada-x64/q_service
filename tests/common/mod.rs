use bevy::log::LogPlugin;
use bevy::prelude::*;
use q_service::prelude::*;

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq)]
pub enum TestErr {
    #[error("A")]
    A,
}
impl ServiceError for TestErr {}

service!(TestService, (), TestErr);
service!(TestService2, (), TestErr);
service!(TestService3, (), TestErr);

pub fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "debug".into(),
            ..Default::default()
        },
    ))
    .add_systems(Startup, || debug!("STARTUP"))
    .add_systems(Update, || debug!("UPDATE"));
    app
}
