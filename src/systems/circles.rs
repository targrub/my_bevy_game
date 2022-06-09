use rand::Rng;
use bevy::math::Vec2;
use bevy::math::Vec3;
use bevy::math::Quat;
use bevy::render::color::Color;
use palette::{FromColor, Hsl/*, Srgb */};
use bevy::ecs::system::Commands;
use bevy::render::camera::OrthographicCameraBundle;
use bevy::transform::components::Transform;
use bevy::ecs::component::Component;
use bevy::ecs::system::Query;
use bevy::ecs::query::With;

use bevy_prototype_lyon::prelude::*;

use crate::SCREEN_WIDTH;

const MIN_RADIUS:f32 = 4.0;
const MAX_CIRCLES_PER_RADIUS:u32 = 100;

#[derive(Component)]
pub struct ExampleShape;

#[derive(Debug)]
struct MyCircle {
    pos: Vec2,
    r: f32,
    c: Color
}

fn intersects_any(c:&MyCircle, cv:&Vec<MyCircle>) -> bool {
    for tc in cv {
        let distsq: f32 = (c.pos.x - tc.pos.x) * (c.pos.x - tc.pos.x) + (c.pos.y - tc.pos.y) * (c.pos.y - tc.pos.y);
        let radsumsq:f32 = (c.r + tc.r) * (c.r + tc.r);
        if (radsumsq + 50.0) > distsq {
            return true
        }
    }
    false
}

fn rand_circle_color(circle_hsl: &mut Hsl, rng:  & mut rand::prelude::ThreadRng) -> Color {
    //let clamped_hue: f32 = num::clamp(circle_hsl.hue.to_degrees() + rng.gen_range(-30.0..30.0), 0.0, 360.0);
    let clamped_hue = rng.gen_range(0.0..30.0);
    circle_hsl.hue = palette::RgbHue::from_degrees(clamped_hue);
    let c_srgb = palette::Srgb::from_color(*circle_hsl);
    Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
}

fn rand_circle_color_variation(circle_hsl: &mut Hsl, rng: &mut rand::prelude::ThreadRng) -> Color {
    circle_hsl.saturation = num::clamp(circle_hsl.saturation + rng.gen_range(-0.01..0.01), 0.0, 1.0);
    circle_hsl.lightness = num::clamp(circle_hsl.lightness + rng.gen_range(-0.05..0.05), 0.3, 0.9);
    let c_srgb = palette::Srgb::from_color(*circle_hsl);
    Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
}

pub fn setup_shape_rendering(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    let mut circs:Vec<MyCircle> = Vec::new();
    let mut r = 20.0;

    let mut rng = rand::thread_rng();

    let mut color_change_count = 0;
    let mut circle_hsl: Hsl = Hsl::new(0.1, 0.8, 0.7);
    let mut current_color = rand_circle_color(&mut circle_hsl, &mut rng);

    let mut circles_of_this_radius: u32 = 0;

    loop {
        let mut success:bool = false;
        for _ in 1..=100 {  // take many chances to fit this circle in
            let npos: Vec2 = Vec2::new(
                rng.gen::<f32>() * (SCREEN_WIDTH as f32 - r * 2.0) + r - SCREEN_WIDTH as f32 / 2.0,
                rng.gen::<f32>() * (SCREEN_WIDTH as f32 - r * 2.0) + r - SCREEN_WIDTH as f32 / 2.0);
            let nc = MyCircle {
              pos : npos,
                r : r,
                c : current_color
            };
            if !intersects_any(&nc, &circs) {
                circs.push(nc);
                success = true;
                circles_of_this_radius += 1;
                break
            }
        }
        // if failure, decrease radius and loop if not <= min_radius
        if !success || circles_of_this_radius >= MAX_CIRCLES_PER_RADIUS {
            circles_of_this_radius = 0;
            r -= 1.0;
            if r <= MIN_RADIUS {
                break
            }
        } else {
            // if success, might change color's hue
            color_change_count += 1;
            if color_change_count >= 30 {    // every 30 circles, change color
                color_change_count = 0;
                current_color = rand_circle_color(&mut circle_hsl, &mut rng);
            } else {
                current_color = rand_circle_color_variation(&mut circle_hsl, &mut rng);
            }
        }
    }

    for c in circs.iter() {
        let circ = shapes::Circle {
            radius: c.r,
            center: Vec2::ZERO,
        };

        commands.spawn_bundle(GeometryBuilder::build_as(
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

pub fn rotate_colors(mut query: Query<&mut DrawMode, With<ExampleShape>>)
{
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
