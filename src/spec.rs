use crate::{deps::IsServiceDep, prelude::*};
use bevy_platform::prelude::*;

/// Used to specify a new service.
#[derive(Debug)]
pub struct ServiceSpec<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    pub(crate) _handle: ServiceHandle<T, D, E>,
    /// The service's type-erased dependencies.
    pub(crate) deps: Vec<Box<dyn IsServiceDep>>,
    /// Does this service begin immediately?
    /// Note that this will run as soon as the first commands are run.
    /// This may be before Startup!
    pub(crate) is_startup: bool,
    /// The data provided on startup.
    pub(crate) initial_data: Option<D>,
    /// Lifecycle hook
    pub(crate) on_init: Option<InitFn<T, D, E>>,
    /// Lifecycle hook
    pub(crate) on_enable: Option<EnableFn<T, D, E>>,
    /// Lifecycle hook
    pub(crate) on_disable: Option<DisableFn<T, D, E>>,
    /// Lifecycle hook
    pub(crate) on_update: Option<UpdateFn<T, D, E>>,
    /// Lifecycle hook
    pub(crate) on_failure: Option<FailureFn<T, D, E>>,
}
macro_rules! on {
    ($($name:ident),*) => {
        $crate::paste::paste! {
            $(
                /// Hook for this lifecycle stage.
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
    on!(Init, Enable, Disable, Failure);

    /// Does this service begin on startup? By default, a service will be lazily
    /// initialized whenever its state is set to
    /// [ServiceState::Enabled], or
    /// when manually initialized with
    /// [Commands::init_service](crate::lifecycle::commands::ServiceLifecycleCommands::init_service)
    /// or the [InitService](crate::lifecycle::events::InitService) event.
    pub fn is_startup(self, is_startup: bool) -> Self {
        Self { is_startup, ..self }
    }
    /// Add dependencies. Dependencies are anything which implement
    /// [IsServiceDep].
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
    /// ```rust, skip
    /// app.add_service(
    ///     ExampleService::spec()
    ///     .with_deps(vec![
    ///         MyDep::spec(),
    ///         MyOtherDep::spec(),
    ///         MyNonServiceDep,
    ///     ]));
    /// ```
    pub fn with_deps(self, deps: Vec<impl IsServiceDep + 'static>) -> Self {
        let deps = deps
            .into_iter()
            .map(|d| Box::new(d) as Box<dyn IsServiceDep>)
            .collect();
        Self { deps, ..self }
    }
    /// Insert data to be available on initialization.
    /// This can be any data type. When this data type is altered, it will
    /// trigger the [on_data_update] event. It can be updated with
    /// [Commands::update_service_data] or the [UpdateServiceData] event.
    /// (TODO!)
    ///
    /// ## Example usage
    /// ```rust,skip
    /// app.add_service(
    ///     ExampleService::spec()
    ///         .with_data(MyData {
    ///             /*...*/
    ///         })
    /// );
    /// ```
    pub fn with_data(self, data: D) -> Self {
        Self {
            initial_data: Some(data),
            ..self
        }
    }
}
