//! Demonstrates how to use the fly camera

extern crate amethyst;
#[macro_use]
extern crate amethyst_derive;
#[macro_use]
extern crate serde;

use amethyst::ecs::WriteStorage;
use amethyst::core::Transform;
use amethyst::assets::AssetPrefab;
use amethyst::assets::Prefab;
use amethyst::utils::removal::Removal;
use amethyst::assets::PrefabData;
use amethyst::ecs::Entity;
use amethyst::assets::PrefabError;
use amethyst::assets::ProgressCounter;
use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat, SubPrefab},
    controls::FlyControlBundle,
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    utils::{application_root_dir},
    Error,
};

#[derive(Deserialize, Serialize)]
pub struct MyPrefabData {
    removal: Option<Removal<i32>>,
    transform: Option<Transform>,
    sub: Option<SubPrefab<MyPrefabData, RonFormat>>,
}

impl Default for MyPrefabData {
    fn default() -> Self {
        MyPrefabData {
            removal: None,
            transform: None,
            sub: None,
        }
    }
}

impl<'a> PrefabData<'a> for MyPrefabData {
    type SystemData = (
        PrefabLoader<'a, MyPrefabData>,
        <Transform as PrefabData<'a>>::SystemData,
        WriteStorage<'a, Removal<i32>>,
    );

    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), amethyst::assets::Error> {
        let (
            ref mut sub_loader,
            ref mut transforms,
            ref mut removals,
        ) = system_data;
        //self.graphics.add_to_entity(entity, graphics, entities)?;
        self.transforms.add_to_entity(entity, removals, entities)?;
        if let Some(i) = self.removal {
            removals.insert(entity, Removal::new(i));
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        let (
            ref mut sub_loader,
            ref mut transforms,
            ref mut removals,
        ) = system_data;
        if self.graphics.load_sub_assets(progress, graphics)? {
            ret = true;
        }
        if self.transform.load_sub_assets(progress, transforms)? {
            ret = true;
        }
        Ok(ret)
    }
}


struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/nested1.ron", RonFormat, (), ())
        });
        data.world
            .create_entity()
            .with(prefab_handle)
            .build();
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();
    let resources_directory = format!("{}/examples/assets", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?;
    let mut game = Application::build(resources_directory, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}
