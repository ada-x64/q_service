use crate::prelude::*;
use bevy_ecs::prelude::*;

/// Run condition which checks if the given service has the given state.
pub fn service_has_state<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
    state: ServiceState<E>,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        service.state == state
    })
}

macro_rules! run_conditions {
    ($($state:ident),*) => {
        $crate::paste::paste! {
            $(
                /// Run condition
                pub fn [<service_ $state:snake:lower>]<T, D, E>(
                    _handle: ServiceHandle<T,D,E>,
                ) -> impl Condition<()>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    IntoSystem::into_system(
                        move |service: Res<Service<T, D, E>>| {
                            matches!(service.state, ServiceState::[<$state:camel>])
                        },
                    )
                }
            )*
        }
    };
}

run_conditions!(Uninitialized, Initializing, Enabled, Disabled);

/// Run condition
pub fn service_failed<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        matches!(service.state, ServiceState::Failed(_))
    })
}

/// Run condition
pub fn service_failed_with_error<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
    error: ServiceErrorKind<E>,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        if let ServiceState::Failed(e) = &service.state {
            *e == error
        } else {
            false
        }
    })
}
