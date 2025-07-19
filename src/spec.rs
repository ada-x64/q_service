use crate::{deps::ServiceDep, prelude::*};
use bevy_platform::prelude::*;

/// Used to specify a new service.
#[derive(Debug)]
pub struct ServiceSpec<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    pub(crate) _handle: ServiceHandle<T, D, E>,
    pub(crate) deps: Vec<ServiceDep>,
    pub(crate) is_startup: bool,
    pub(crate) initial_data: Option<D>,
    pub(crate) on_init: Option<InitFn<T, D, E>>,
    pub(crate) on_enable: Option<EnableFn<T, D, E>>,
    pub(crate) on_disable: Option<DisableFn<T, D, E>>,
    pub(crate) on_update: Option<UpdateFn<T, D, E>>,
    pub(crate) on_failure: Option<FailureFn<T, D, E>>,
}
macro_rules! on {
    ($(( $name:ident, $doc:expr )),*) => {
        $crate::paste::paste! {
            $(
                #[doc=$doc]
                pub fn [<on_ $name:snake:lower>]<M>(self, s: impl [<Into $name:camel Fn>]<T,D,E, M>) -> Self {
                    Self {
                        [< on_ $name:snake:lower >]: Some([<$name:camel Fn>]::new(s)),
                        ..self
                    }
                }
            )*
        }
    };
}

impl<T, D, E> Default for ServiceSpec<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn default() -> Self {
        Self {
            _handle: ServiceHandle::const_default(),
            deps: vec![],
            is_startup: false,
            initial_data: None,
            on_init: None,
            on_enable: None,
            on_disable: None,
            on_failure: None,
            on_update: None,
        }
    }
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> ServiceSpec<T, D, E> {
    // Hook setters.
    on!(
        (
            Init,
            "Hook for the initialization stage.
            Allows you to perform actions before the service is fully initialized.
            The return value of this function determines whether the service is \
            enabled or disabled once initializtion is complete."
        ),
        (
            Enable,
            "Hook for the enable stage.
            Allows you to perform actions before the service is enabled."
        ),
        (
            Disable,
            "Hook for the disable stage.
            Allows you to perform actions before the service is disabled."
        ),
        (
            Failure,
            "Hook for reacting to failures.
            Allows you to perform actions before the service is set to Failed."
        ),
        (
            Update,
            "Hook for reacting to data updates.
            Allows you to transform the data before it is stored in the service."
        )
    );

    /// Does this service begin on startup? By default, a service will be lazily
    /// initialized whenever its state is set to
    /// [ServiceState::Enabled], or
    /// when manually initialized with
    /// [Commands::init_service](crate::lifecycle::commands::ServiceLifecycleCommands::init_service)
    /// or the [InitService](crate::lifecycle::events::InitService) event.
    pub fn is_startup(self, is_startup: bool) -> Self {
        Self { is_startup, ..self }
    }
    /// Add dependencies. A dependency can be an [Asset](bevy_asset::Asset),
    /// [Resource](bevy_ecs::resource::Resource), or another [Service].
    ///
    /// When this service initializes, it will recursively initialize all of its
    /// dependencies. If any of the dependencies fail, this service will fail to
    /// initialize.
    ///
    /// ## Panics
    ///
    /// If there are any cycles in the dependencies, the specified service will
    /// panic on add.
    ///
    /// ## Example usage
    /// ```rust
    /// # use q_service::prelude::*;
    /// # use bevy_app::App;
    /// #
    /// # #[derive(ServiceError, thiserror::Error, Debug, PartialEq, Clone)]
    /// # pub enum MyError {}
    /// #
    /// service!(MyService, (), MyError);
    /// service!(MyDep, (), MyError);
    ///
    /// pub fn main() {
    ///     let mut app = App::new();
    ///     app.add_service(
    ///         MyService::default_spec()
    ///         .with_deps(vec![
    ///             MyDep::handle().into(),
    ///         ])
    ///     );
    /// }
    /// ```
    pub fn with_deps(self, deps: Vec<ServiceDep>) -> Self {
        Self { deps, ..self }
    }
    /// Insert data to be available on initialization.
    /// This can be any data type. When this data type is altered, it will
    /// trigger the [ServiceHooks::on_update] event. It can be updated with
    /// [Commands::update_service](crate::lifecycle::commands::ServiceLifecycleCommands::update_service) or the [UpdateService](crate::lifecycle::events::UpdateService) event.
    ///
    /// ## Example usage
    /// ```rust
    /// # use q_service::prelude::*;
    /// # use bevy_app::App;
    /// #
    /// # #[derive(ServiceError, thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
    /// # pub enum MyError {}
    /// # #[derive(ServiceData, Debug, Default, Clone, PartialEq)]
    /// # pub struct MyData {}
    /// #
    /// service!(MyService, MyData, MyError);
    /// // ...
    /// # fn main() {
    ///     let mut app = App::new();
    ///     app.add_service(
    ///         MyService::default_spec()
    ///             .with_data(MyData {
    ///                 /*...*/
    ///             })
    ///         );
    /// # }
    /// ```
    pub fn with_data(self, data: D) -> Self {
        Self {
            initial_data: Some(data),
            ..self
        }
    }
}
