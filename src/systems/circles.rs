use bevy::app::App;
use bevy::asset::{Assets, Handle};
use bevy::core_pipeline::{clear_color::ClearColorConfig, core_2d::Camera2d};
use bevy::ecs::world::{FromWorld, World};
use bevy::ecs::{
    component::Component,
    system::{Commands, Query, Res, ResMut, SystemParam},
};
use bevy::math::{Vec2, Vec3};
use bevy::render::{camera::Camera, color::Color, mesh::shape, mesh::Mesh, view::RenderLayers};
use bevy::sprite::{ColorMaterial, Material2d, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::utils::default;
use palette::{rgb::Rgb, FromColor, Hsl, Srgb};
use rand::Rng;
use std::cmp::Ordering;
//use bevy::prelude::*;

use crate::systems::color_generator;
use crate::StartColor;

use super::dynamic_textures::RenderToTextureDescriptor;

const MIN_RADIUS: f32 = 4.0;
const MAX_CIRCLES_PER_RADIUS: u32 = 100;

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

pub fn circles1_add_circles_to_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<&mut Circles1>,
) {
    if query.is_empty() {
        return;
    }
    for mut circles1 in &mut query {
        if circles1.done_setup {
            continue;
        }
        let first_pass_layer = RenderLayers::layer(circles1.layer);
        let window_width = 1280; //windows.primary().physical_width();

        let mut circs: Vec<MyCircle> = Vec::new();
        let mut r = 20.0;

        let mut rng = rand::thread_rng();

        let (mut generator, mut current_color) = color_generator::ColorGenerator::new(
            circles1.start_color.hue,
            circles1.start_color.saturation,
            circles1.start_color.lightness,
        );

        let mut color_change_count = 0;
        let mut circles_of_this_radius: u32 = 0;

        loop {
            let mut success: bool = false;
            for _ in 1..=100 {
                // take many chances to fit this circle in
                let npos: Vec2 = Vec2::new(
                    rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r
                        - window_width as f32 / 2.0,
                    rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r
                        - window_width as f32 / 2.0,
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
        circles1.done_setup = true;
    }
}

//#[allow(clippy::manual_swap)]
pub fn circles1_update_colors(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<&mut Circles1>,
) {
    if query.is_empty() {
        return;
    }
    for circles1 in &mut query {
        if !circles1.done_setup {
            continue;
        }
        // get the color of the last circle in the list
        let (saved_handle_id, mut saved_color_mat) = materials.iter_mut().last().unwrap();
        let mut prev_color = saved_color_mat.color;
        // rotate all the colors around through all ColorMaterials
        for m in materials.iter_mut() {
            std::mem::swap(&mut m.1.color, &mut prev_color);
        }
    }
}

//------------------------------------------------------

struct AllCircles {
    pos: Vec<Vec2>,
    r: Vec<f32>,
    c: Vec<Color>,
}

impl AllCircles {
    fn new() -> Self {
        Self {
            pos: Vec::new(),
            r: Vec::new(),
            c: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct Circles1 {
    pub layer: u8,
    pub start_color: StartColor,
    pub background_color: Color,
    done_setup: bool,
}

impl Circles1 {
    pub fn new(layer: u8, desc: &RenderToTextureDescriptor) -> Circles1 {
        Circles1 {
            layer,
            start_color: desc.start_color,
            background_color: desc.background_color,
            done_setup: false,
        }
    }
    // drop() ... gets rid of setup/update systems
}

#[derive(Component)]
pub struct Circles2 {
    pub layer: u8,
    pub start_color: StartColor,
    pub background_color: Color,
    allcircs: AllCircles,
    done_setup: bool,
}

impl Circles2 {
    pub fn new(layer: u8, desc: &RenderToTextureDescriptor) -> Circles2 {
        Circles2 {
            layer,
            start_color: desc.start_color,
            background_color: desc.background_color,
            allcircs: AllCircles::new(),
            done_setup: false,
        }
    }
}

pub fn circles2_add_circles_to_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<&mut Circles2>,
) {
    if query.is_empty() {
        return;
    }
    for mut circles2 in &mut query {
        if circles2.done_setup {
            continue;
        }
        let first_pass_layer = RenderLayers::layer(circles2.layer);
        let window_width = 1280; //windows.primary().physical_width();

        let mut r = 20.0;

        let mut rng = rand::thread_rng();

        let (mut generator, mut current_color) = color_generator::ColorGenerator::new(
            circles2.start_color.hue,
            circles2.start_color.saturation,
            circles2.start_color.lightness,
        );

        let mut color_change_count = 0;
        let mut circles_of_this_radius: u32 = 0;

        loop {
            let mut success: bool = false;
            for _ in 1..=100 {
                // take many chances to fit this circle in
                let npos: Vec2 = Vec2::new(
                    rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r
                        - window_width as f32 / 2.0,
                    rng.gen::<f32>() * (window_width as f32 - r * 2.0) + r
                        - window_width as f32 / 2.0,
                );
                if !intersects_any2(npos, r, &circles2.allcircs.pos, &circles2.allcircs.r) {
                    circles2.allcircs.pos.push(npos);
                    circles2.allcircs.r.push(r);
                    circles2.allcircs.c.push(current_color);
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

        for (pos, (r, c)) in circles2
            .allcircs
            .pos
            .iter()
            .zip(circles2.allcircs.r.iter().zip(circles2.allcircs.c.iter()))
        {
            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(*r).into()).into(),
                    material: materials.add(ColorMaterial::from(*c)),
                    transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
                    ..default()
                })
                .insert(first_pass_layer);
        }
        circles2.done_setup = true;
    }
}

pub fn circles2_update(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<&mut Circles2>,
    mut query2: Query<(&mut Mesh2dHandle, &mut Transform)>,
    //    mut query3: Query<&mut ColorMaterial>,
) {
    if query.is_empty() || query2.is_empty() {
        return;
    }
    let t = time.time_since_startup().as_secs_f32();
    for circles2 in &mut query {
        if !circles2.done_setup {
            continue;
        }
        for (r, (mut m, _)) in circles2.allcircs.r.iter().zip(&mut query2) {
            m.0 = meshes.add(shape::Circle::new(*r * (1.0 + 0.12 * (10.0 * *r * t).sin())).into());
        }
        for (p, (_, mut tr)) in circles2.allcircs.pos.iter().zip(&mut query2) {
            *tr = Transform::from_translation(Vec3::new(
                p.x + 5.0 * (0.7 * p.x * t).tan().abs().clamp(0.0, 1.0),
                p.y + 3.0 * (3.1 * p.y * t).sin().abs().clamp(0.0, 1.0),
                0.0,
            ));
        }
        for (c, m) in circles2.allcircs.c.iter().zip(materials.iter_mut()) {
            let mut hsl = Hsl::from_color(Rgb::new(c.r(), c.g(), c.b()));
            hsl.saturation = num::clamp(hsl.saturation + 0.4 * (3.0 * t).sin(), 0.0, 1.0);
            hsl.lightness = num::clamp(hsl.lightness + 0.4 * (5.0 * t).sin(), 0.3, 0.9);
            let c_srgb = Srgb::from_color(hsl);
            m.1.color = Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0);
        }
        // camera2dbundle.camera_2d.clear_color = ClearColorConfig::Custom(background_color)
    }
}

fn intersects_any2(pos: Vec2, r: f32, vec_pos: &[Vec2], vec_radius: &[f32]) -> bool {
    for (tpos, tr) in vec_pos.iter().zip(vec_radius) {
        let distsq: f32 = (pos.x - tpos.x) * (pos.x - tpos.x) + (pos.y - tpos.y) * (pos.y - tpos.y);
        let radsumsq: f32 = (r + tr) * (r + tr);
        if (radsumsq + 50.0) > distsq {
            return true;
        }
    }
    false
}
