use crate::prelude::*;
use bevy_derive::*;
use bevy_ecs::prelude::*;

macro_rules! trigger_lifecycle_events {
    ($(($name:ident, $doc:expr $(, $err:ty)*)$(,)?)*) => {
        $crate::paste::paste! {
            $(
                #[doc = $doc]
                #[derive(Event)]
                pub struct [<$name Service>]<T, D, E>(pub ServiceHandle<T, D, E> $(, pub $err)*)
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError;
            )*
        }
    }
}
trigger_lifecycle_events!(
    (
        Enable,
        "Triggers the service's Enable lifecycle event. See [ServiceHooks::on_enable] for more info."
    ),
    (
        Disable,
        "Triggers the service's Disable lifecycle event. See [ServiceHooks::on_disable] for more info."
    ),
    (
        Init,
        "Triggers the service's Init lifecycle event.
        The event will give a warning if the service is already initialized.
        See [ServiceHooks::on_init] for more info."
    ),
    (
        Fail,
        "Triggers the service's Fail lifecycle event. This will shut down the service.
        See [ServiceHooks::on_failure] for more info.",
        ServiceErrorKind<E>
    ),
    (
        Update,
        "Triggers the service's Update lifecycle event. This changes the data stored in the service.
        See [ServiceHooks::on_update] for more info.",
        D
    )
);

macro_rules! state_change {
    ( $( ($name:ident, $($ss:ty)+)$(,)?)* ) => {
        $(
            #[allow(missing_docs)]
            #[derive(Event, Deref)]
            pub struct $name<T, D, E>(
                #[deref]
                $(pub $ss)*,
                ServiceHandle<T,D,E>
            )
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
                #[allow(missing_docs)]
                pub fn new(val: $($ss)*) -> Self {
                    Self(val, ServiceHandle::const_default())
                }
                #[allow(missing_docs)]
                pub fn new_with_handle(handle: ServiceHandle<T,D,E>, val: $($ss)*) -> Self {
                    Self(val, handle)
                }
            }
        )*
    };
}
state_change!(
    (ServiceStateChange, (ServiceState<E>, ServiceState<E>)),
    (ExitServiceState, ServiceState<E>),
    (EnterServiceState, ServiceState<E>),
);

macro_rules! enter_state_aliases {
    ($(($name:ident, $doc:expr $(, $err_ty:ty )*)$(,)?)*) => {
        $crate::paste::paste! {
            $(
                #[doc=$doc]
                #[allow(dead_code, reason="macro")]
                #[derive(Event)]
                pub struct [<Service $name>]<T, D, E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    _handle: ServiceHandle<T, D, E>,
                    $(err: $err_ty)*
                }
                impl<T, D, E> [<Service $name>]<T, D, E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    #[allow(missing_docs)]
                    pub fn new(_handle: ServiceHandle<T, D, E>, $(err: $err_ty)*) -> Self {
                        Self { _handle, $(err: err as $err_ty)* }
                    }
                }
            )*
        }
    };
}

enter_state_aliases!(
    (Enabled, "Fires when the service becomes enabled."),
    (Disabled, "Fires when the service becomes disabled."),
    (
        Initialized,
        "Fires when the service finishes initialization."
    ),
    (
        Updated,
        "Fires when service data has been updated.
        To access the data, add `service: Res<MyService>` to your system
        parameters and call `service.data()`. To transform the data, use the [on_update hook.](crate::prelude::ServiceHooks::on_update)"
    ),
    (
        Failed,
        "Fires when the service has failed. Reports the error kind.",
        ServiceErrorKind<E>
    )
);
