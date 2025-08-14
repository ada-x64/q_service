use crate::{prelude::*, spec::ServiceSpec};
use bevy_app::prelude::*;
use bevy_asset::{Asset, AssetPath, DirectAssetAccessExt};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};

/// Used to scope systems, resources, and assets to a service.
pub struct ServiceScope<'a, T: Service> {
    app: &'a mut App,
    spec: ServiceSpec<T>,
}
impl<'a, T: Service> ServiceScope<'a, T> {
    pub(crate) fn new(app: &'a mut App) -> Self {
        Self {
            app,
            spec: ServiceSpec::default(),
        }
    }
    pub(crate) fn into_spec(self) -> ServiceSpec<T> {
        self.spec
    }
    /// Adds systems to this service.
    /// Will automatically scope these systems so that they run only if the service is up.
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel + Clone,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.app
            .add_systems(schedule.clone(), systems.in_set(T::system_set()));
        self.app
            .configure_sets(schedule, T::system_set().run_if(service_up::<T>()));
        self
    }

    /// Adds an initialization function to the service.
    /// The init hook may return a task to be polled. If so, the service
    /// will remain in the Initializing state until the task finishes.
    ///
    /// # Example usage
    /// ```rust
    /// # let app = App::new();
    /// # let scope = ServiceScope::new(&mut app);
    /// fn my_default_init() -> InitResult {
    ///     Ok(None)
    /// }
    /// spec.with_init(my_default_init);
    ///
    /// fn my_async_init() -> InitResult {
    ///     let task = AsyncHook::async_compute_task(async |_| {
    ///         // do something async here
    ///         Ok(())
    ///     });
    ///     Ok(Some(task))
    /// }
    /// spec.with_init(my_async_init);
    /// ```
    pub fn init_with<M>(&mut self, system: impl IntoInitHook<T, M>) -> &mut Self {
        self.spec.on_init = Some(InitHook::new(system));
        self
    }

    /// Adds a deinitialization function to the service.
    /// The deinit hook may return a task to be polled. If so, the service
    /// will remain in the Deinitializing state until the task finishes.
    ///
    /// # Example usage
    /// ```rust
    /// # let app = App::new();
    /// # let scope = ServiceScope::new(&mut app);
    /// fn my_default_deinit() -> DeinitResult {
    ///     Ok(None)
    /// }
    /// spec.with_deinit(my_default_init);
    ///
    /// fn my_async_deinit() -> DeinitResult {
    ///     let task = AsyncHook::async_compute_task(async |_| {
    ///         // do something async here
    ///         Ok(())
    ///     });
    ///     Ok(Some(task))
    /// }
    /// spec.with_deinit(my_async_init);
    /// ```
    pub fn deinit_with<M>(&mut self, system: impl IntoDeinitHook<T, M>) -> &mut Self {
        self.spec.on_deinit = Some(DeinitHook::new(system));
        self
    }

    /// Adds a hook which will run when the service is up.
    ///
    /// ## Example usage
    /// ```rust
    /// # let app = App::new();
    /// # let scope = ServiceScope::new(&mut app);
    /// fn my_up_hook() -> UpResult {
    ///     Ok(())
    /// }
    /// spec.on_up(my_up_hook);
    /// ```
    pub fn on_up<M>(&mut self, system: impl IntoUpHook<T, M>) -> &mut Self {
        self.spec.on_up = Some(UpHook::new(system));
        self
    }

    /// Adds a hook which will run when the service is down.
    ///
    /// ## Example usage
    /// ```rust
    /// # let app = App::new();
    /// # let scope = ServiceScope::new(&mut app);
    /// fn my_down_hook(reason: In<DownReason>) {
    ///     match reason {
    ///         DownReason::Uninitialized => todo!(),
    ///         DownReason::Failed(service_error_kind) => todo!(),
    ///         DownReason::SpunDown => todo!(),
    ///     }
    /// }
    /// spec.on_up(my_down_hook);
    /// ```
    pub fn on_down<M>(&mut self, system: impl IntoDownHook<T, M>) -> &mut Self {
        self.spec.on_down = Some(DownHook::new(system));
        self
    }

    /// Adds the given service as a dependency.
    /// Make sure this dependency is also registered, or you'll run into errors!
    pub fn add_dep<S: Service>(&mut self) -> &mut Self {
        self.app.init_resource::<S>();
        let cid = self
            .app
            .world()
            .resource_id::<S>()
            .expect("Resource id should exist");
        let id = NodeId::Service(cid);
        let data = ServiceData::new::<S>(cid);
        self.app
            .world_mut()
            .resource_mut::<GraphDataCache>()
            .entry(id)
            .or_insert(GraphData::Service(data));
        self.spec.deps.push(id);
        self
    }

    /// Adds a resource to this service, initializing with its Default value.
    /// The resource will be instantiated when the service is spun up, and
    /// removed when the service is spun down.
    pub fn add_resource<R: Resource + Default>(&mut self) -> &mut Self {
        self.add_resource_with(R::default);
        self
    }

    /// Adds a resource to this service with a custom default value.
    /// The resource will be instantiated when the service is spun up, and
    /// removed when the service is spun down.
    pub fn add_resource_with<R: Resource, M>(
        &mut self,
        default: impl IntoSystem<(), R, M> + 'static,
    ) -> &mut Self {
        let world = self.app.world_mut();
        let init_sys = default.pipe(|input: In<R>, mut commands: Commands| {
            commands.insert_resource(input.0);
        });
        let init = world.register_system(init_sys).entity();
        let deinit = world
            .register_system(|mut commands: Commands| {
                commands.remove_resource::<R>();
            })
            .entity();
        // registers resource without inserting it into the world
        let id = world.register_resource::<R>();
        let data = GraphData::resource::<R>(world, init, deinit);
        world
            .resource_mut::<GraphDataCache>()
            .insert(NodeId::Resource(id), data);
        self.spec.deps.push(NodeId::Resource(id));
        self
    }

    /// Adds an asset to the service. The asset will be load a strong handle
    /// into an entity which will stay alive as long as the service is up. So,
    /// the asset added here will live _at least_ as long as the service.
    pub fn add_asset<A: Asset>(&mut self, path: impl Into<AssetPath<'a>>) -> &mut Self {
        let world = self.app.world_mut();
        let handle = world.load_asset::<A>(path.into());
        let id = handle.id().untyped();
        let data = GraphData::asset::<A, T>(handle, world);
        world
            .resource_mut::<GraphDataCache>()
            .insert(NodeId::Asset(id), data);
        self.spec.deps.push(NodeId::Asset(id));
        self
    }

    /// Does this service spin up at startup?
    /// Defaults to false.
    pub fn is_startup(&mut self, val: bool) -> &mut Self {
        self.spec.is_startup = val;
        self
    }
}
