use crate::prelude::*;
use bevy_ecs::prelude::*;
use tracing::*;

macro_rules! command_trait {
    ($( ($name:ident $(, $err:ty )*)$(,)?)*) => {
        /// Extends [Commands] with service-related functionality.
        pub trait ServiceLifecycleCommands {
            $crate::paste::paste! {
                $(
                    #[allow(missing_docs, reason="macro")]
                    fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E> $(, error: $err)*)
                        where
                            T: ServiceLabel,
                            D: ServiceData,
                            E: ServiceError;
                )*
            }
        }
        impl<'w, 's> ServiceLifecycleCommands for Commands<'w, 's> {
            $crate::paste::paste! {
                $(
                    fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E> $(, err: $err)*)
                        where
                            T: ServiceLabel,
                            D: ServiceData,
                            E: ServiceError,
                    {
                        self.queue([<$name:camel Service>]::<T, D, E>::new(handle $(, err as $err)*));
                    }
                )*
            }
        }
    };
}
command_trait!((Init), (Enable), (Disable), (Fail, ServiceErrorKind<E>));

macro_rules! commands {
    ($(( $name:ident, $fn:ident $(, $err:ty)* )$(,)?)+) => {
        $(
        pub(crate) struct $name<T, D, E>(ServiceHandle<T, D, E> $(, $err)*)
        where
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError;
        impl<T, D, E> $name<T, D, E>
        where
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError,
        {
            pub fn new(handle: ServiceHandle<T,D,E> $(, err: $err)*) -> Self {
                Self(handle $(, err as $err)*)
            }
        }

        impl_command!($name, $fn $(, $err)*);
        )+
    };
}

macro_rules! impl_command {
    ($name:ident, $fn:ident $(, $err:ty)*) => {
        impl<T, D, E> Command for $name<T, D, E>
        where
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError,
        {
            fn apply(self, world: &mut World) {
                if world.get_resource::<Service<T,D,E>>().is_none() {
                    return warn!("Tried to get missing service. Did you try calling a hook within a hook? If so, prefer reacting to service state changes.");
                }
                world.resource_scope(
                    |world, mut service: Mut<Service<T, D, E>>| {
                        let _ = service.$fn(world, $(self.1 as $err, false)*);
                    },
                )
            }
        }
    };
}

commands!(
    (InitService, on_init),
    (EnableService, on_enable),
    (DisableService, on_disable),
    (FailService, on_failure, ServiceErrorKind<E>)
);
