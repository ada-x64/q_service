use std::marker::PhantomData;

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Tick,
    system::{ReadOnlySystemParam, SystemMeta, SystemParam},
    world::{Mut, World, unsafe_world_cell::UnsafeWorldCell},
};

use crate::prelude::*;

/// SystemParam for convenient access to services.
#[derive(Deref)]
pub struct ServiceRef<'a, T: Service> {
    #[deref]
    service: &'a ServiceData,
    _handle: PhantomData<T>,
}

unsafe impl<'a, T: Service> SystemParam for ServiceRef<'a, T> {
    type State = ();

    type Item<'world, 'state> = ServiceRef<'world, T>;

    fn init_state(_: &mut World, _: &mut SystemMeta) -> Self::State {}

    unsafe fn get_param<'world, 'state>(
        _: &'state mut Self::State,
        _: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        _: Tick,
    ) -> Self::Item<'world, 'state> {
        let world = unsafe { world.world() };
        let service = world.service::<T>();
        Self::Item {
            service,
            _handle: PhantomData,
        }
    }
}
unsafe impl<'a, T: Service> ReadOnlySystemParam for ServiceRef<'a, T> {}

/// SystemParam for convenient mutable access to services.
#[derive(Deref, DerefMut)]
pub struct ServiceMut<'a, T: Service> {
    #[deref]
    service: Mut<'a, ServiceData>,
    _handle: PhantomData<T>,
}

unsafe impl<'a, T: Service> SystemParam for ServiceMut<'a, T> {
    type State = ();

    type Item<'world, 'state> = ServiceMut<'world, T>;

    fn init_state(_: &mut World, _: &mut SystemMeta) -> Self::State {}

    unsafe fn get_param<'world, 'state>(
        _: &'state mut Self::State,
        _: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        _: Tick,
    ) -> Self::Item<'world, 'state> {
        let world = unsafe { world.world_mut() };
        let service = world.service_mut::<T>();
        Self::Item {
            service,
            _handle: PhantomData,
        }
    }
}
