use bevy::prelude::*;
use bevy_asset::AssetLoader;
use q_service::prelude::*;

mod common;
use common::*;

#[derive(thiserror::Error, Debug)]
enum TestAssetError {}

#[derive(Asset, Reflect)]
struct TestAsset;

struct TestAssetLoader;
impl AssetLoader for TestAssetLoader {
    type Asset = TestAsset;

    type Settings = ();

    type Error = TestAssetError;

    fn load(
        &self,
        _reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy_asset::LoadContext,
    ) -> impl bevy_tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        async {
            debug!("(Test asset) Loading ...");
            busy_wait(500);
            debug!("(Test asset) ... Done!");
            Ok(TestAsset)
        }
    }
}

#[derive(Resource, Debug, Default)]
struct AssetDep;
impl Service for AssetDep {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.is_startup(true).add_asset::<TestAsset>("test.txt");
    }
}

#[test]
fn asset_dep() {
    let mut app = setup();
    app.init_asset::<TestAsset>()
        .register_asset_loader(TestAssetLoader)
        .register_service::<AssetDep>();
    app.update();
    app.world_mut()
        .service_scope::<AssetDep, _>(|world, service| {
            assert!(service.status().is_initializing());
            service.deps().iter().for_each(|dep| {
                if let Some(asset) = world.resource::<GraphDataCache>().get_asset(*dep) {
                    assert!(asset.status.is_initializing());
                }
            });
        });
    busy_wait(1000); // wait extra long for CI
    app.update();
    app.world_mut()
        .service_scope::<AssetDep, _>(|world, service| {
            assert!(service.status().is_up());
            service.deps().iter().for_each(|dep| {
                if let Some(asset) = world.resource::<GraphDataCache>().get_asset(*dep) {
                    assert!(asset.status.is_up());
                }
            });
        });
    app.world_mut().commands().spin_service_down::<AssetDep>();
    app.update();
    app.world_mut()
        .service_scope::<AssetDep, _>(|world, service| {
            assert!(service.status().is_down());
            service.deps().iter().for_each(|dep| {
                if let Some(asset) = world.resource::<GraphDataCache>().get_asset(*dep) {
                    assert!(asset.status.is_down());
                }
            });
        });
}

#[test]
fn persistent_asset() {
    let mut app = setup();
    app.init_asset::<TestAsset>()
        .register_asset_loader(TestAssetLoader)
        .register_service::<AssetDep>();
    app.update();
    busy_wait(1000); // wait extra long for CI
    let mut handle = None;
    app.world_mut()
        .service_scope::<AssetDep, _>(|world, service| {
            let dep_cache = world.resource::<GraphDataCache>();
            service.deps().iter().for_each(|dep| {
                if let Some(asset) = dep_cache.get_asset(*dep) {
                    handle = Some(
                        world
                            .entity(asset.container)
                            .get::<KeepHandleAlive<TestAsset>>()
                            .unwrap()
                            .0
                            .clone(),
                    );
                }
            });
        });
    app.world_mut().commands().spin_service_down::<AssetDep>();
    app.update();
    app.world_mut()
        .service_scope::<AssetDep, _>(|world, service| {
            assert!(service.status().is_down());
            service.deps().iter().for_each(|dep| {
                if let Some(asset) = world.resource::<GraphDataCache>().get_asset(*dep) {
                    assert!(asset.status.is_down());
                }
            });
        });
    let _asset = app
        .world()
        .resource::<Assets<TestAsset>>()
        .get(handle.unwrap().id())
        .unwrap();
}
