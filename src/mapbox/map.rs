// TODO: of course, we will use Trait

use super::config;
use super::manifest_observer::ManifestObserver;
use super::style::Style;
use super::vector_tile_manager::VectorTileManager;
use super::vector_tile_observer::VectorTileObserver;

use super::common::types::{Threadable, ThreadableNew};

pub struct Map {
    map_impl: Threadable<MapImpl>,
    style: Style,
}

impl Map {
    pub fn new() -> Map {
        Map {
            map_impl: ThreadableNew(MapImpl::new()),

            style: Style::new(),
        }
    }

    pub fn load_map(&mut self, stylesheet_url: &'static str) {
        let obs = self.map_impl.clone();
        self.style.add_manifest_observer(obs);
        self.style.load_style_with_url(stylesheet_url);
    }

    pub fn set_center(&mut self, lat: f32, long: f32) {
        // TODO: do some bounds checking
        self.map_impl.lock().unwrap().set_center(lat, long);
    }

    pub fn get_center_point_tile_xy(&self) -> (f32, f32) {
        self.map_impl.lock().unwrap().get_center_point_tile_xy()
    }

    pub fn get_covered_tiles_coords(&self) -> Vec<(f32, f32)> {
        self.map_impl.lock().unwrap().get_covered_tiles_coords()
    }

    pub fn get_zoom(&self) -> f32 {
        self.map_impl.lock().unwrap().get_zoom()
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        // TODO: do some bounds checking
        self.map_impl.lock().unwrap().set_zoom(zoom);
    }

    // User Interactions

    pub fn pan(&mut self, _delta_lat: f64, _delta_long: f64) {
        // TODO: do some bounds checking
    }

    pub fn zoom(&mut self, _delta_zoom: f64) {
        // TODO: do some bounds checking
    }

    // Observsers

    pub fn add_vector_tile_observer(
        &mut self,
        vector_tile_obs: Threadable<dyn VectorTileObserver>,
    ) {
        self.map_impl
            .lock()
            .unwrap()
            .add_vector_tile_observer(vector_tile_obs);
    }
}

// ManifestObserver

struct MapImpl {
    zoom: f32,
    center: (f32, f32), // (lat, long) of the map center
    vector_tiles: VectorTileManager,
}

impl MapImpl {
    pub fn new() -> MapImpl {
        MapImpl {
            zoom: config::MAP_DEFAULT_ZOOM_LEVEL,
            center: (0.0, 0.0),
            vector_tiles: VectorTileManager::new(),
        }
    }

    pub fn get_center_point_tile_xy(&self) -> (f32, f32) {
        self.vector_tiles
            .get_center_point_tile_xy(self.center, self.zoom)
    }

    pub fn get_covered_tiles_coords(&self) -> Vec<(f32, f32)> {
        let screen_size = (
            crate::config::GL_VIEWPORT_WIDTH,
            crate::config::GL_VIEWPORT_HEIGHT,
        );
        let covered_tile_ids =
            self.vector_tiles
                .get_covered_tiles(&self.center, self.zoom, &screen_size);
        covered_tile_ids
            .iter()
            .map(|v| (v.x as f32, v.y as f32))
            .collect()
    }

    pub fn set_center(&mut self, lat: f32, long: f32) {
        // TODO: do some bounds checking
        self.center = (lat, long);
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        // TODO: do some bounds checking
        self.zoom = zoom;
        self.vector_tiles.set_zoom(zoom);
    }

    fn load_tiles(&mut self, vector_name: String, url_template: String) {
        let screen_size = (
            crate::config::GL_VIEWPORT_WIDTH,
            crate::config::GL_VIEWPORT_HEIGHT,
        );
        self.vector_tiles.load_covered_tiles(
            vector_name,
            &self.center,
            self.zoom,
            screen_size,
            url_template,
        );
    }

    pub fn add_vector_tile_observer(
        &mut self,
        vector_tile_obs: Threadable<dyn VectorTileObserver>,
    ) {
        self.vector_tiles.add_vector_tile_observer(vector_tile_obs);
    }
}

impl ManifestObserver for MapImpl {
    fn on_manifest_loaded(&mut self, name: String, url_template: String, avail_zooms: Vec<u64>) {
        let current_zoom = self.get_zoom();
        println!(
            "== Manifest URL: {}, Curr Zoom: {}, Avail zooms: {:?}",
            url_template, current_zoom, avail_zooms
        );
        if avail_zooms.contains(&(current_zoom as u64)) {
            self.load_tiles(name.to_uppercase(), url_template);
        }
    }

    fn on_manifest_failed(&self, name: String) {
        println!("===== map heard it failed {}", name);
    }
}
