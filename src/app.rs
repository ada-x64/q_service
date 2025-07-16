use crate::prelude::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use tracing::*;

macro_rules! events {
    ($app:ident, $($name:ident $(,)?)* ) => {
        $(
            $app.add_event::<$name<T, D, E>>();
        )*
    }
}

macro_rules! observers {
    ($app:ident, $( ( $name:ident $(, $err:ty )* )$(,)?)*) => {
        $(
            $crate::paste::paste! {
                $app.add_observer(
                    |trigger: Trigger<$name<T, D, E>>,
                     mut commands: Commands| {
                         commands.[<$name:snake:lower>](trigger.event().0.clone(), $(trigger.event().1.clone() as $err)*);
                    },
                );
            }
        )*
    };
}

#[allow(missing_docs)]
pub trait ServiceExt<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    /// Add a service to the application.
    ///
    /// This function takes in a [ServiceSpec], which should be specified using
    /// the [service!] macro from this crate.
    ///
    /// ## Example usage
    /// ```rust, skip
    /// service!(Example, (), ExampleErr);
    /// app.add_service(ExampleService::spec());
    /// ```
    /// ## Panics
    /// This function panics if cycles are detected in the ServiceSpec's
    /// dependencies.
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self;
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> ServiceExt<T, D, E>
    for App
{
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self {
        debug!("Adding service {}", std::any::type_name::<T>());

        // no dupes
        if self.world().get_resource::<Service<T, D, E>>().is_some() {
            warn!(
                "Tried to add already existing service {:?}",
                std::any::type_name::<T>()
            );
            return self;
        }

        // Register events
        use crate::lifecycle::events::{
            DisableService, EnableService, FailService, InitService,
        };

        events!(
            self,
            EnterServiceState,
            ExitServiceState,
            EnableService,
            DisableService,
            InitService,
            FailService,
        );
        observers!(
            self,
            (EnableService),
            (DisableService),
            (InitService),
            (FailService, ServiceErrorKind<E>),
        );

        let world = self.world_mut();
        let is_startup = spec.is_startup;

        // Add resource
        world.insert_resource(Service::from_spec(spec));

        // Initialize on startup
        if is_startup {
            world.schedule_scope(Startup, |_world, sched| {
                debug!("{} will initialize at startup.", std::any::type_name::<T>());
                sched.add_systems(move |mut commands: Commands| {
                    commands.init_service(ServiceHandle::<T, D, E>::const_default());
                });
            });
        }
        self
    }
}
