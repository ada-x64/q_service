use crate::prelude::*;
use bevy_ecs::prelude::*;

/// Run condition which checks if the given service has the given state.
pub fn service_has_status<T>(target_status: ServiceStatus) -> impl Condition<()>
where
    T: Service,
{
    IntoSystem::into_system(move |service: ServiceRef<T>| service.status() == target_status)
}

macro_rules! run_conditions {
    ($(( $state:ident, $doc:tt )),*) => {
        $crate::paste::paste! {
            $(
                #[doc=$doc]
                pub fn [<service_ $state:snake:lower>]<T>() -> impl Condition<()>
                where
                    T: Service,
                {
                    IntoSystem::into_system(
                        move |service: ServiceRef<T>| {
                            service.status().[<is_ $state:snake:lower>]()
                        },
                    )
                }
            )*
        }
    };
}

run_conditions!(
    (Up, "Run condition. Is the service up?"),
    (Down, "Run condition. Is the service down?"),
    (
        Initializing,
        "Run condition. Is the service initializing? Note: If the service
        initializes synchronously, or if init takes less than a frame, then this
        will never fire."
    ),
    (
        Deinitializing,
        "Run condition. Is the service deinitializing? Note: If the service
        deinitializes synchronously, or if deinit takes less than a frame, then
        this will never fire."
    )
);

/// Run condition. Has the service failed? Will fire on any [ServiceError].
pub fn service_failed<T>() -> impl Condition<()>
where
    T: Service,
{
    IntoSystem::into_system(move |service: ServiceRef<T>| {
        matches!(service.status(), ServiceStatus::Down(DownReason::Failed(_)))
    })
}

/// Run condition. Has the service failed? Will fire only on the specified [ServiceError].
pub fn service_failed_with_error<T>(error: ServiceError) -> impl Condition<()>
where
    T: Service,
{
    IntoSystem::into_system(move |service: ServiceRef<T>| match service.status() {
        ServiceStatus::Down(DownReason::Failed(ref e)) => e == &error,
        _ => false,
    })
}
