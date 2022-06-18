use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
//use bevy::ecs::system::Res;
use bevy::math::Quat;
use bevy::math::Vec2;
use bevy::math::Vec3;
use bevy::render::camera::OrthographicCameraBundle;
use bevy::render::color::Color;
use bevy::transform::components::Transform;
//use bevy::window::Windows;
use rand::Rng;

use bevy_prototype_lyon::prelude::*;

use crate::systems::color_generator;

const MIN_RADIUS: f32 = 4.0;
const MAX_CIRCLES_PER_RADIUS: u32 = 100;

#[derive(Component)]
pub struct ExampleShape;

#[derive(Debug)]
struct MyCircle {
    pos: Vec2,
    r: f32,
    c: Color,
}

fn intersects_any(c: &MyCircle, cv: &[MyCircle]) -> bool {
    for tc in cv {
        let distsq: f32 = (c.pos.x - tc.pos.x) * (c.pos.x - tc.pos.x)
            + (c.pos.y - tc.pos.y) * (c.pos.y - tc.pos.y);
        let radsumsq: f32 = (c.r + tc.r) * (c.r + tc.r);
        if (radsumsq + 50.0) > distsq {
            return true;
        }
    }
    false
}

pub fn setup_shape_rendering(mut commands: Commands/* , windows: Res<Windows>*/) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window_width = 1280;//windows.primary().physical_width();

    let mut circs: Vec<MyCircle> = Vec::new();
    let mut r = 20.0;

    let mut rng = rand::thread_rng();

    let (mut generator, mut current_color) = color_generator::ColorGenerator::new(0.1, 0.8, 0.7);

    let mut color_change_count = 0;
    let mut circles_of_this_radius: u32 = 0;

    loop {
        let mut success: bool = false;
        for _ in 1..=100 {
            // take many chances to fit this circle in
            let npos: Vec2 = Vec2::new(
                rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r - window_width as f32 / 2.0,
                rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r - window_width as f32 / 2.0,
            );
            let nc = MyCircle {
                pos: npos,
                r,
                c: current_color,
            };
            if !intersects_any(&nc, &circs) {
                circs.push(nc);
                success = true;
                circles_of_this_radius += 1;
                break;
            }
        }
        // if failure, decrease radius and loop if not <= min_radius
        if !success || circles_of_this_radius >= MAX_CIRCLES_PER_RADIUS {
            circles_of_this_radius = 0;
            r -= 1.0;
            if r <= MIN_RADIUS {
                break;
            }
        } else {
            // if success, might change color's hue
            color_change_count += 1;
            if color_change_count >= 30 {
                // every 30 circles, change color
                color_change_count = 0;
                current_color = generator.rand_color(&mut rng);
            } else {
                current_color = generator.rand_color_variation(&mut rng);
            }
        }
    }

    for c in circs.iter() {
        let circ = shapes::Circle {
            radius: c.r,
            center: Vec2::ZERO,
        };

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &circ,
                DrawMode::Fill(FillMode::color(c.c)),
                Transform {
                    translation: Vec3::new(c.pos.x, c.pos.y, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            ))
            .insert(ExampleShape);
    }
}

#[allow(clippy::manual_swap)]
pub fn rotate_colors(mut query: Query<&mut DrawMode, With<ExampleShape>>) {
    // get the color of the last circle in the list
    let mut prev_drawmode = DrawMode::Fill(FillMode::color(Color::BLACK));
    for mode in query.iter() {
        prev_drawmode = *mode;
    }

    // iterate through entities, and change entity n's color to that of entity n-1
    for mut mode in query.iter_mut() {
        let save_drawmode = *mode;
        *mode = prev_drawmode;
        prev_drawmode = save_drawmode;
    }
}
