use std::marker::PhantomData;

use crate::prelude::*;
use bevy_app::{App, PostStartup, PreUpdate, Startup};
use bevy_ecs::{component::ComponentId, prelude::*};
use tracing::{debug, warn};

macro_rules! register_parameterized_events {
    ($app:ident, $($name:ident $(,)?)* ) => {
        $(
            $app.add_event::<$name<Self>>();
        )*
    }
}

/// A trait for resources which wrap [ServiceData] instances. You can think of
/// services as a kind of dynamic plugin which can be spun up or down at
/// runtime. See the [top-level docs](crate) for more details.
pub trait Service: Resource + Sized + std::fmt::Debug + Default {
    /// Registers systems and service behavior using a [ServiceScope]. The
    /// services will only run if the system is up. Service dependencies will be
    /// automatically spun up and down with the parent service. Resources and
    /// assets will automatically be fetched when the service spins up, and
    /// removed when the service spins down.
    ///
    /// If you want to scope other systems to this service's states, see
    /// [crate::run_conditions] and the [system_set](Service::system_set) function.
    ///
    /// ## Example usage
    /// ```rust
    /// # use q_service::prelude::*;
    /// # use bevy::prelude::*;
    /// # service!(ExampleService);
    /// #
    /// # fn main() {
    /// # let mut app = App::new();
    /// # fn sys_a() {}
    /// # fn sys_b() {}
    /// fn build(scope: ServiceScope<ExampleService>) {
    ///     scope.add_dep::<MyDep>();
    ///     scope.add_systems(Update, (sys_a, sys_b).chain());
    ///     scope.add_resource::<MyResource>();
    ///     scope.add_asset::<MyAsset>(asset_path);
    /// }
    /// # }
    /// ```
    fn build(scope: &mut ServiceScope<Self>);

    /// Gets the display name for this service.
    fn name() -> String {
        name_from_type::<Self>()
    }

    /// Creates and instantiates the service wrapper,
    /// inserting it as a resource in the world.
    #[tracing::instrument(skip_all)]
    fn register(app: &mut App) {
        debug!("({}) Registering...", Self::name(),);

        // no dupes
        if let Some(t) = app.world().get_resource::<Self>()
            && t.data(app.world()).registered()
        {
            warn!("Overriding already registered service {}", Self::name());
        }
        register_parameterized_events!(
            app,
            // set state
            LifecycleCommand,
            // react to state
            EnterServiceState,
            ExitServiceState,
            ServiceStateChange,
            ServiceInitializing,
            ServiceDeinitializing,
            ServiceUp,
            ServiceDown,
        );
        app.add_event::<ServiceUpdated>();

        // ensure dependencies
        app.init_resource::<DependencyGraph>();
        app.init_resource::<GraphDataCache>();
        app.init_resource::<Self>();

        let id = app.world().resource_id::<Self>().unwrap();
        let system_set = LifecycleSystems(id);
        let set = (
            || debug!("({}) Running PostUpdate Service Lifecycle", Self::name()),
            watch_service_commands::<Self>,
            poll_tasks::<Self>,
            update_dep_status::<Self>,
            update_async_state::<Self>,
            broadcast_new_state::<Self>,
        )
            .chain()
            .in_set(system_set);
        app.add_systems(PreUpdate, set);

        let set = (
            || debug!("({}) Running PostStartup Service Lifecycle", Self::name()),
            watch_service_commands::<Self>,
            poll_tasks::<Self>,
            update_dep_status::<Self>,
            update_async_state::<Self>,
            broadcast_new_state::<Self>,
        )
            .chain()
            .in_set(system_set);
        app.add_systems(PostStartup, set);

        // make spec
        let mut scope = ServiceScope::new(app);
        Self::build(&mut scope);
        let spec = scope.into_spec();

        // run dep lifecycles in order to keep status propogation stable
        for dep in spec.deps.iter() {
            if let NodeId::Service(id) = dep {
                app.configure_sets(PreUpdate, system_set.after(LifecycleSystems(*id)));
                app.configure_sets(PostStartup, system_set.after(LifecycleSystems(*id)));
            }
        }

        if spec.is_startup {
            app.add_systems(Startup, move |mut commands: Commands| {
                commands.spin_service_up::<Self>();
            });
        }

        // Instantiate service and cache it
        // If this already exists it will be overwritten. This is what we want,
        // when we delcare a service wrapper we're defining the canoncial implementation.
        ServiceData::register::<Self>(app.world_mut(), spec);
        debug!("({}) ...Done!", Self::name(),);
    }

    /// Fetches the underlying service data.
    fn data<'w>(&self, world: &'w World) -> &'w ServiceData {
        let cache = world.resource::<GraphDataCache>();
        let id = NodeId::Service(world.resource_id::<Self>().unwrap());
        cache.get_service(id).unwrap()
    }

    /// Fetches the underlying service data.
    fn data_mut<'w>(&self, world: &'w mut World) -> Mut<'w, ServiceData> {
        let id = NodeId::Service(world.resource_id::<Self>().unwrap());
        world
            .resource_mut::<GraphDataCache>()
            .map_unchanged(|cache| cache.get_service_mut(id).unwrap())
    }

    /// Returns the [SystemSet] associated with this service.
    fn system_set() -> ServiceSystems<Self> {
        ServiceSystems::<Self>(PhantomData)
    }
}

/// A [SystemSet] associated to a specific [Service]. Sytems in this set will
/// only run when the service is up.
#[derive(SystemSet)]
pub struct ServiceSystems<T: Service>(PhantomData<T>);

impl<T: Service> std::fmt::Debug for ServiceSystems<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ServiceSystems").field(&self.0).finish()
    }
}

impl<T: Service> Copy for ServiceSystems<T> {}

impl<T: Service> Clone for ServiceSystems<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Service> PartialEq for ServiceSystems<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Service> Eq for ServiceSystems<T> {}

impl<T: Service> std::hash::Hash for ServiceSystems<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A [SystemSet] associated to a specific [Service]. Sytems in this set will
/// only run when the service is up.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct LifecycleSystems(ComponentId);
