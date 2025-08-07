use bevy::prelude::*;
use q_service::prelude::*;

#[derive(Event)]
pub struct SomeEvent;

#[derive(Resource, Debug, Default)]
pub struct MyOtherService;
impl Service for MyOtherService {
    fn build(_: &mut ServiceScope<MyOtherService>) {}
}

#[derive(Resource, Debug, Default)]
pub struct SomeOtherService;
impl Service for SomeOtherService {
    fn build(_: &mut ServiceScope<SomeOtherService>) {}
}

#[derive(Resource, Default)]
pub struct MyResource;

#[derive(Asset, Reflect)]
pub struct MyAsset;

pub fn sys_a() {}
pub fn sys_b() {}
