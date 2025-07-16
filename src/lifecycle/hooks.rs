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
                pub struct [<$name Fn>]<T,D,E>(
                    #[deref]
                    Box<dyn System<In = $in, Out = $out>>,
                    ServiceHandle<T,D,E>
                ) where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError;

                impl<T,D,E> [<$name Fn>]<T,D,E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError
                {
                    #[allow(missing_docs)]
                    pub fn new<M, S: IntoSystem<$in, $out, M>>(s: S) -> Self {
                        Self(Box::new(IntoSystem::into_system(s)), ServiceHandle::const_default())
                    }
                }

                impl<T,D,E> Default for [<$name Fn>]<T,D,E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError
                {
                    fn default() -> Self {
                        Self::new($default)
                    }
                }
                #[allow(missing_docs)]
                pub trait [<Into $name Fn>]<T,D,E, M>:
                    IntoSystem<$in, $out, M>
                    where
                        T: ServiceLabel,
                        D: ServiceData,
                        E: ServiceError
                {
                }
                impl<T, D, E, M, S> [<Into $name Fn>]<T, D, E, M> for S where
                    S: IntoSystem<$in, $out, M>,
                    T: ServiceLabel, D: ServiceData, E: ServiceError
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
    (Update, In<D>, Result<D, E>, |d: In<D>| Ok(d.clone())),
);

/// Contains hooks for the given service. See module-level documentation for
/// details.
#[derive(Debug)]
pub struct ServiceHooks<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    /// Hook which executes on initialization. Will forward to
    /// [on_enable](Self::on_enable) or [on_disable](Self::on_disable) when
    /// finished.
    pub on_init: InitFn<T, D, E>,
    /// Hook which executes on enable. Will initialize if needed.
    pub on_enable: EnableFn<T, D, E>,
    /// Hook which executes on disable. Will warn if uninitialized.
    pub on_disable: DisableFn<T, D, E>,
    /// Hook which executes when the stored data is changed. This executes _before_ the data has been updated, giving you the chance to transform it.
    /// To react to data changes _after_ they have been updated, use [OnServiceUpdate]
    // TODO: There should be a difference between the input type to this function and the interally stored data type.
    pub on_update: UpdateFn<T, D, E>,
    /// Hook which executes on failure.
    pub on_failure: FailureFn<T, D, E>,
}
macro_rules! on {
    ($($name:ident),*) => {
        $crate::paste::paste! {
            $(
                #[allow(missing_docs)]
                pub fn [<on_ $name:snake:lower>]<S, M>(self, s: S) -> Self
                where
                    S: [<Into $name:camel Fn>]<T, D, E, M> // "Tedium"
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

impl<T, D, E> ServiceHooks<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    on!(Init, Enable, Disable, Failure, Update);
}
// note: E is not Default so can't derive this
impl<T, D, E> Default for ServiceHooks<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn default() -> Self {
        Self {
            on_init: InitFn::default(),
            on_enable: EnableFn::default(),
            on_disable: DisableFn::default(),
            on_failure: FailureFn::default(),
            on_update: UpdateFn::default(),
        }
    }
}
