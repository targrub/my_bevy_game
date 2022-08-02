use bevy::asset::{Assets, Handle};
use bevy::core_pipeline::{clear_color::ClearColorConfig, core_2d::Camera2d};
use bevy::ecs::{component::Component, {system::{Commands, Query, Res, ResMut}}};
use bevy::math::{Vec2, Vec3};
use bevy::render::{camera::Camera, color::Color, mesh::shape, mesh::Mesh, view::RenderLayers};
use bevy::sprite::{ColorMaterial, Material2d, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::transform::components::Transform;
use bevy::utils::default;
use rand::Rng;
use std::cmp::Ordering;

use crate::StartColor;
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

pub fn add_circles_to_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    start_color: &StartColor,
    background_color: &Color,
    layer_num: u8,
) {
    let first_pass_layer = RenderLayers::layer(layer_num);
    let window_width = 1280; //windows.primary().physical_width();

    let mut circs: Vec<MyCircle> = Vec::new();
    let mut r = 20.0;

    let mut rng = rand::thread_rng();

    let (mut generator, mut current_color) = color_generator::ColorGenerator::new(
        start_color.hue,
        start_color.saturation,
        start_color.lightness,
    );

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

    for c in &circs {
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(c.r).into()).into(),
                material: materials.add(ColorMaterial::from(c.c)),
                transform: Transform::from_translation(Vec3::new(c.pos.x, c.pos.y, 0.0)),
                ..default()
            })
            .insert(first_pass_layer);
    }
}

//#[allow(clippy::manual_swap)]
pub fn update_colors(mut meshes: &mut Assets<Mesh>, mut materials: &mut Assets<ColorMaterial>) {
    // get the color of the last circle in the list
    let (saved_handle_id, mut saved_color_mat) = materials.iter_mut().last().unwrap();
    let mut prev_color = saved_color_mat.color;
    // rotate all the colors around through all ColorMaterials
    for m in materials.iter_mut() {
        std::mem::swap(&mut m.1.color, &mut prev_color);
    }
}
