use ncollide2d::shape::{Ball, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionObject, CollisionObjectHandle, 
    CollisionWorld, GeometricQueryType};

use amethyst::ecs::{Component, DenseVecStorage, Entity};
use nalgebra::Isometry2 as Isometry;

lazy_static! {
    static ref HITBOX_GROUP: CollisionGroups = CollisionGroups::new();
}

pub struct HitboxComponent {
    hitbox: Option<CollisionObjectHandle>,
}

impl HitboxComponent {
    fn new(world: &mut CollisionWorld<f32, Entity>, entity: Entity, box_rad: f32) 
        -> Self 
    {
        let hitbox = world.add(
            Isometry::<f32>::identity(),
            ShapeHandle::new(Ball::new(box_rad)),
            *HITBOX_GROUP,
            GeometricQueryType::Proximity(40.0_f32),
            entity
        );
        HitboxComponent {
            hitbox: Option::Some(hitbox)
        }
    }
}

impl Component for HitboxComponent {
    type Storage = DenseVecStorage<Self>;
}

pub struct BoidAIStateComponent {
    panic_level: f32,
}

impl Component for BoidAIStateComponent {
    type Storage = DenseVecStorage<Self>;
}