use crate::prelude::*;
use bevy_ecs::prelude::*;
use tracing::*;

macro_rules! command_trait {
    ($( ($name:ident, $doc:expr $(, $data:ty )* $(,)?)$(,)?)*) => {
        /// Extends [Commands] with service-related functionality.
        pub trait ServiceLifecycleCommands {
            $crate::paste::paste! {
                $(
                    #[doc=$doc]
                    fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E> $(, data: $data)*)
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
                    fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E> $(, data: $data)*)
                        where
                            T: ServiceLabel,
                            D: ServiceData,
                            E: ServiceError,
                    {
                        self.queue([<$name:camel Service>]::<T, D, E>::new(handle $(, data as $data)*));
                    }
                )*
            }
        }
    };
}
command_trait!(
    (
        Init,
        "Directly initializes the service. See [module-level docs](crate::lifecycle) for more info.",
    ),
    (
        Enable,
        "Directly enables the service. See [module-level docs](crate::lifecycle) for more info.",
    ),
    (
        Disable,
        "Directly disables the service. See [module-level docs](crate::lifecycle) for more info.",
    ),
    (
        Fail,
        "Directly fails the service. This will shut the service down. See [module-level docs](crate::lifecycle) for more info.",
        ServiceErrorKind<E>
    ),
    (
        Update,
        "Directly updates the service. This calls the update hook, potentially transforming the input data before storing it in the service.
        See [module-level docs](crate::lifecycle) for more info.",
        D
    )
);

macro_rules! commands {
    ($(( $name:ident, $fn:ident $(, ($input_name:ident : $input_ty: ty))?))*) => {
        $(
        pub(crate) struct $name<T, D, E>(ServiceHandle<T, D, E> $(, $input_ty)*)
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
            pub fn new(handle: ServiceHandle<T,D,E> $(, $input_name : $input_ty)?) -> Self {
                Self(handle $(, $input_name)*)
            }
        }

        impl_command!($name, $fn $(, ($input_name: $input_ty))?);
        )+
    };
}

macro_rules! impl_command {
    ($name:ident, $fn:ident $(, ($input_name:ident : $input_ty: ty ))?) => {
        impl<T, D, E> Command for $name<T, D, E>
        where
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError,
        {
            fn apply(self, world: &mut World) {
                if world.get_resource::<Service<T,D,E>>().is_none() {
                    let mut msg = "Tried to get missing service.".to_string();
                    msg += "\n.. Did you try calling a hook within a hook?\n.. If so, prefer using service state change events.";
                    msg += "\n.. Did you forget to register your service?\n.. If so, make sure to call `app.add_service(MyService::defuault_spec())`.";
                    return warn!("{msg}");
                }
                world.resource_scope(
                    |world, mut service: Mut<Service<T, D, E>>| {
                        let _ = service.$fn(world $(, self.1 as $input_ty)?);
                    },
                )
            }
        }
    };
}

commands!(
    (InitService, on_init)
    (EnableService, on_enable)
    (DisableService, on_disable)
    (FailService, on_fail_cmd, (error: ServiceErrorKind<E>))
    (UpdateService, on_update, (data: D))
);
