#![feature(trace_macros)]

extern crate amethyst;
extern crate ncollide2d;
extern crate nalgebra;
extern crate rand;
#[macro_use]
extern crate stackbt;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod decide;
mod components;
mod systems;

use amethyst::prelude::*;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::{Vector3, Matrix4, Quaternion};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::renderer::{
    Camera, DisplayConfig, DrawFlat, Event, KeyboardInput, MaterialTextureSet,
    Pipeline, PngFormat, PosTex, Projection, RenderBundle, Stage, Sprite, 
    SpriteSheet, SpriteSheetHandle, SpriteRenderData, Texture, TextureHandle, 
    VirtualKeyCode, WithSpriteRender, WindowEvent, 
};
use amethyst::ecs::Entity;
use ncollide2d::world::CollisionWorld;

pub struct BoidFields;

const SPRITESHEET_SIZE: (f32, f32) = (16.0, 16.0);
const FIELD_HEIGHT: f32 = 768.0;
const FIELD_WIDTH: f32 = 1280.0;

const BOID_SIDE: f32 = 16.0;

impl<'a, 'b> State<GameData<'a, 'b>> for BoidFields {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        world.add_resource(CollisionWorld::<f32, Entity>::new(0.1_f32));
        let sprite_sheet_handle = load_sprite_sheet(world);

        initialise_boids(world, &sprite_sheet_handle);
        initialise_camera(world);
    }


    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn initialise_camera(world: &mut World) {
    world.create_entity()
        .with(Camera::from(Projection::orthographic(
            -FIELD_WIDTH/2.0,
            FIELD_WIDTH/2.0,
            FIELD_HEIGHT/2.0,
            -FIELD_HEIGHT/2.0,
        )))
        .with(GlobalTransform(
            Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0))
        ))
        .build();
}

fn load_sprite_sheet(world: &mut World) -> TextureHandle {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        "./stackbt/examples/resources/boid.png",
        PngFormat,
        Default::default(),
        (),
        &texture_storage,
    )
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_boids(world: &mut World, sprite_sheet: &TextureHandle) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = FIELD_HEIGHT / 2.0;
    left_transform.translation = Vector3::new(BOID_SIDE/2.0_f32, y, 0.0);
    right_transform.translation = Vector3::new(FIELD_WIDTH - BOID_SIDE/2.0_f32, y, 0.0);
    right_transform.rotation = Quaternion::<f32>::from_arc(
        Vector3::new(1.0_f32, 0.0_f32, 0.0_f32),
        Vector3::new(-1.0_f32, 0.0_f32, 0.0_f32),
        Option::None
    );

    // Build the sprite for the paddles.
    let sprite = Sprite {
        left: 0.0,
        right: BOID_SIDE,
        top: 0.0,
        bottom: BOID_SIDE,
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with_sprite(&sprite, sprite_sheet.clone(), SPRITESHEET_SIZE)
        .expect("Failed to add sprite render on left paddle")
        .with(GlobalTransform::default())
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with_sprite(&sprite, sprite_sheet.clone(), SPRITESHEET_SIZE)
        .expect("Failed to add sprite render on left paddle")
        .with(GlobalTransform::default())
        .with(right_transform)
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = "./stackbt/examples/resources/display_config.ron";
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );
    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_bundle(TransformBundle::new())?;
    let mut game = Application::new("./", BoidFields, game_data)?;
    game.run();
    Ok(())
}
