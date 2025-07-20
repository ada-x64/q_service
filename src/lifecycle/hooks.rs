use crate::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use std::marker::PhantomData;

macro_rules! hooks {
    ($(($name: ident, in = $in:ty, out = $out:ty, $doc:literal $(,)?)),* $(,)?) => {
        $crate::paste::paste! {
            $(
                #[doc = $doc]
                #[doc = "\n\nWraps a function F where F: [`IntoSystem`] with In = [`"]
                #[doc = stringify!($in)]
                #[doc = "`], Out =[`"]
                #[doc = stringify!($out)]
                #[doc = "`]"]
                #[derive(Deref, DerefMut, Debug)]
                pub struct [< $name Hook>]<T>(
                    #[deref]
                    pub(crate) Box<dyn System<In = $in, Out = $out>>,
                    PhantomData<T>
                ) where
                    T: Service;

                impl<T> [< $name Hook>]<T>
                where
                    T: Service,
                {
                    #[allow(missing_docs)]
                    pub fn new<M, S: IntoSystem<$in, $out, M>>(s: S) -> Self {
                        Self(Box::new(IntoSystem::into_system(s)), PhantomData::default())
                    }
                }

                #[allow(missing_docs)]
                pub trait [<Into $name Hook>]<T,M>:
                    IntoSystem<$in, $out, M>
                    where
                        T: Service,
                    {
                }
                impl<T, M, S> [<Into $name Hook>]<T, M> for S where
                    S: IntoSystem<$in, $out, M>,
                    T: Service,
                {
                }
            )*
        }
    };
}

hooks!(
    (
        Init,
        in = (),
        out = InitResult,
        "A [Service]'s initialization function. Use this to do whatever is needed to bring the service up."
    ),
    (
        Up,
        in = (),
        out = UpResult,
        "Runs when the [Service] changes state to Up. Must be synchronous."
    ),
    (
        Deinit,
        in = (),
        out = DeinitResult,
        "A [Service]'s deinitialization function. Use this to do whatever is needed to bring the service down."
    ),
    (
        Down,
        in = In<DownReason>,
        out = (),
        "Runs when the [Service] changes state to Down. Must be synchronous."
    ),
);

/// The result returned from the Init hook.
pub type InitResult = Result<Option<AsyncHook>, BevyError>;
/// The result returned from the Deinit hook.
pub type DeinitResult = Result<Option<AsyncHook>, BevyError>;
/// The result retunred from the Up hook.
pub type UpResult = Result<(), BevyError>;
