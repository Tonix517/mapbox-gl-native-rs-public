// TODO: use trait for projectors

use super::super::config;
use std::f64::consts;

pub struct Tranformation {}

impl Tranformation {
    pub fn latlong_to_tile_coord(lat: f32, long: f32, zoom: u32) -> (f32, f32) {
        let k = (2 as u32).pow(zoom) as f32 / (config::TILE_SIZE as f32);
        (
            Tranformation::long_x(long) * k,
            Tranformation::lat_y(lat) * k,
        )
    }

    fn long_x(lng: f32) -> f32 {
        (180.0 + lng) * (config::TILE_SIZE as f32) / 360.0
    }

    fn lat_y(lat: f32) -> f32 {
        let y_ = 180.0 / consts::PI
            * (consts::PI / 4.0 + (lat as f64) * consts::PI / 360.0)
                .tan()
                .log(consts::E);
        ((180.0 - y_) * (config::TILE_SIZE as f64) / 360.0) as f32
    }
}
