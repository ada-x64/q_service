use crate::prelude::*;
use bevy_derive::*;
use bevy_ecs::prelude::*;
use std::marker::PhantomData;

macro_rules! state_change {
    ( $( ($name:ident, $($ss:ty)+)$(,)?)* ) => {
        $(
            #[allow(missing_docs)]
            #[derive(Event, Deref, Clone, Debug)]
            pub struct $name<T>(
                #[deref]
                $(pub $ss)*,
                PhantomData<T>
            )
            where
                T: Service;

            impl<T> $name<T>
            where
                T: Service,
            {
                #[allow(missing_docs)]
                pub fn new(val: $($ss)*) -> Self {
                    Self(val, PhantomData::default())
                }
            }
        )*
    };
}
state_change!(
    (ServiceStateChange, (ServiceStatus, ServiceStatus)),
    (ExitServiceState, ServiceStatus),
    (EnterServiceState, ServiceStatus),
);

macro_rules! enter_state_aliases {
    ($((
            $name:ident,
            $(
                ($($item_name:ident : $item_ty:ty),*),
                ($($init_param:ident : $init_ty:ty),*),
                ($($field:ident : $initializer:expr),*),
            )?
            $doc:literal $(,)?
        )),*
    ) => {
        $crate::paste::paste! {
            $(
                #[doc=$doc]
                #[doc="This must be called with [`Commands::send_event`]"]
                #[derive(Event, Debug)]
                pub struct [<$name>]<T>
                where
                    T: Service,
                {
                    _handle: PhantomData<T>,
                    $($(
                        #[allow(unused, reason="public api")]
                        $item_name: $item_ty
                    ),*)?
                }
                impl<T> [<$name>]<T>
                where
                    T: Service,
                {
                    #[allow(missing_docs)]
                    #[allow(clippy::new_without_default)]
                    #[allow(unused, reason="It is.")]
                    pub (crate) fn new($($($init_param: $init_ty),*)?) -> Self {
                        Self {
                            _handle: PhantomData::default(),
                            $($($field : $initializer),*)?
                        }
                    }
                }
            )*
        }
    };
}

enter_state_aliases!(
    (
        ServiceInitializing,
        "Fires when the service begins asychronously initializing."
    ),
    (
        ServiceUp,
        "Fires when the service becomes enabled."
    ),
    (
        ServiceDeinitializing,
        (reason: DownReason), (), (reason: DownReason::SpunDown),
        "Fires when the service begins asychronously deinitializing."
    ),
    (
        ServiceDown,
        (reason: DownReason), (), (reason: DownReason::SpunDown),
        "Fires when the service has been spun down.",
    ),
    (
        ServiceFailing,
        (reason: DownReason), (error: ServiceError), (reason: DownReason::Failed(error)),
        "Fires when the service begins failing with an aschronous deinitializer."
    ),
    (
        ServiceFailed,
        (reason: DownReason), (error: ServiceError), (reason: DownReason::Failed(error)),
        "Fires when the service has been spun down due to an error.",
    )
);
