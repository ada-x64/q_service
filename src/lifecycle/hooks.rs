use crate::data::*;
use bevy_derive::*;
use bevy_ecs::prelude::*;
use tracing::*;

macro_rules! hooks {
    ($(($name: ident, $in:ty, $out:ty, $default:expr ),)*) => {
        $crate::paste::paste! {
            $(
                #[allow(missing_docs)]
                #[derive(Deref, DerefMut, Debug)]
                pub struct [<$name Fn>]<E: ServiceError>(
                    Box<dyn System<In = $in, Out = $out>>,
                );
                impl<E: ServiceError> [<$name Fn>]<E> {
                    #[allow(missing_docs)]
                    pub fn new<M, S: IntoSystem<$in, $out, M>>(s: S) -> Self {
                        Self(Box::new(IntoSystem::into_system(s)))
                    }
                }
                impl<E: ServiceError> Default for [<$name Fn>]<E> {
                    fn default() -> Self {
                        Self::new($default)
                    }
                }
                #[allow(missing_docs)]
                pub trait [<Into $name Fn>]<E: ServiceError, M>:
                    IntoSystem<$in, $out, M>
                {
                }
                impl<E: ServiceError, M, T> [<Into $name Fn>]<E, M> for T where
                    T: IntoSystem<$in, $out, M>
                {
                }
            )*
        }
    };
}

hooks!(
    (Init, (), Result<bool, E>, || Ok(true)),
    (Enable, (), Result<(), E>, || Ok(())),
    (Disable, (), Result<(), E>, || Ok(())),
    (Failure, In<ServiceErrorKind<E>>, (), |e: In<ServiceErrorKind<E>>| {error!("Service error: {e:?}");}),
);

/// Contains hooks for the given service. See module-level documentation for
/// details.
#[derive(Debug)]
pub struct ServiceHooks<E>
where
    E: ServiceError,
{
    /// Hook which executes on initialization. Will forward to
    /// [on_enable](Self::on_enable) or [on_disable](Self::on_disable) when
    /// finished.
    pub on_init: InitFn<E>,
    /// Hook which executes on enable. Will initialize if needed.
    pub on_enable: EnableFn<E>,
    /// Hook which executes on disable. Will warn if uninitialized.
    pub on_disable: DisableFn<E>,
    /// Hook which executes on failure.
    pub on_failure: FailureFn<E>,
}
macro_rules! on {
    ($($name:ident),*) => {
        $crate::paste::paste! {
            $(
                #[allow(missing_docs)]
                pub fn [<on_ $name:snake:lower>]<S, M>(self, s: S) -> Self
                where
                    S: [<Into $name:camel Fn>]<E, M>
                {
                    Self {
                        [<on_ $name:snake:lower>]: [<$name Fn>]::new(s),
                        ..self
                    }
                }
            )*
        }
    };
}

impl<E: ServiceError> ServiceHooks<E> {
    on!(Init, Enable, Disable, Failure);
}
// note: E is not Default so can't derive this
impl<E: ServiceError> Default for ServiceHooks<E> {
    fn default() -> Self {
        Self {
            on_init: InitFn::default(),
            on_enable: EnableFn::default(),
            on_disable: DisableFn::default(),
            on_failure: FailureFn::default(),
        }
    }
}
