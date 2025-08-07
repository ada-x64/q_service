use crate::prelude::*;
use bevy_app::prelude::*;

/// Extensions to [App].
pub trait ServiceAppExt {
    /// Add a [Service] to the application.
    ///
    /// ## Example usage
    /// ```rust
    /// # use q_service::prelude::*;
    /// # use bevy::prelude::*;
    /// #[derive(Resource, Debug, Default)]
    /// pub struct ExampleService;
    /// impl Service for ExampleService {
    ///     fn build(scope: &mut ServiceScope) {}
    /// }
    ///
    /// fn main() {
    ///   let mut app = App::new();
    ///   app.register_service::<ExampleService>();
    /// }
    /// ```
    /// ## Panics
    ///
    /// This function panics if cycles are detected in the ServiceSpec's
    /// dependencies.
    fn register_service<T: Service>(&mut self) -> &mut Self;

    // TODO: Dynamic system patching? Probably don't modify hooks.
    // /// Patch a service using a [ServiceScope]. Useful for extending the service's functionality.
    // /// the system is up. For similar use cases when the system is down or in
    // /// another state, see [crate::run_conditions].
    // ///
    // /// ## Example usage
    // /// ```rust
    // /// # use q_service::prelude::*;
    // /// # use bevy::prelude::*;
    // /// # service!(ExampleService);
    // /// #
    // /// # fn main() {
    // /// # let mut app = App::new();
    // /// # fn sys_a() {}
    // /// # fn sys_b() {}
    // /// app.service_scope(|scope: ServiceScope<ExampleService>| {
    // ///     scope.add_systems(Update, (sys_a, sys_b).chain());
    // /// });
    // /// # }
    // /// ```
    // fn service_scope<T: Service>(&mut self, cb: impl FnMut(ServiceScope<T>)) -> &mut Self;
}
impl ServiceAppExt for App {
    fn register_service<T: Service>(&mut self) -> &mut Self {
        T::register(self);
        self
    }
}
