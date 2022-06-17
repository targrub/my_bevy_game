use bevy::render::color::Color;
use rand::Rng;

use palette::{FromColor, Hsl /*, Srgb */};

pub struct ColorGenerator {
    rng: rand::prelude::ThreadRng,
    hsl: palette::Hsl,
}

impl ColorGenerator {
    pub fn new(
        hue: f32,
        saturation: f32,
        lightness: f32,
        rng: rand::prelude::ThreadRng,
    ) -> (ColorGenerator, Color) {
        let hsl = Hsl::new(hue, saturation, lightness);
        let c_srgb = palette::Srgb::from_color(hsl);
        (
            ColorGenerator { hsl, rng },
            Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0),
        )
    }

    pub fn rand_color(&mut self) -> Color {
        //let clamped_hue: f32 = num::clamp(circle_hsl.hue.to_degrees() + rng.gen_range(-30.0..30.0), 0.0, 360.0);
        let clamped_hue: f32 = self.rng.gen_range(0.0..30.0);
        self.hsl.hue = palette::RgbHue::from_degrees(clamped_hue);
        let c_srgb = palette::Srgb::from_color(self.hsl);
        Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
    }

    pub fn rand_color_variation(&mut self) -> Color {
        self.hsl.saturation = num::clamp(
            self.hsl.saturation + self.rng.gen_range(-0.01..0.01),
            0.0,
            1.0,
        );
        self.hsl.lightness = num::clamp(
            self.hsl.lightness + self.rng.gen_range(-0.05..0.05),
            0.3,
            0.9,
        );
        let c_srgb = palette::Srgb::from_color(self.hsl);
        Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
    }
}
