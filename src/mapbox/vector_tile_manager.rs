use super::common::map_error::MapError;
use super::common::task_responder::TaskResponder;
use super::common::types::{Threadable, ThreadableNew};
use super::config;
use super::config::TILE_SIZE;
use super::io::resource::Resource;
use super::utils::pbf::Pbf;
use super::utils::transformation::Tranformation;
use super::vector_tile_id::VectorTileID;
use super::vector_tile_model::VectorTileModel;
use super::vector_tile_observer::VectorTileObserver;

use std::collections::HashMap;
use std::string::ToString;
use std::sync::Arc;

pub struct VectorTileManager {
    vector_tile_manager_impl: Threadable<VectorTileManagerImpl>,
    resource: Resource,
    zoom: f32,
}

impl VectorTileManager {
    pub fn new() -> VectorTileManager {
        let vector_tile_manager_impl = ThreadableNew(VectorTileManagerImpl::new());
        let resource = Resource::new(4);
        VectorTileManager {
            vector_tile_manager_impl,
            resource,
            zoom: config::MAP_DEFAULT_ZOOM_LEVEL,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    pub fn load_covered_tiles(
        &self,
        vector_name: String,
        center_lat_long: &(f32, f32),
        zoom: f32,
        screen_size: (u32, u32),
        url_template: String,
    ) {
        let covered_tiles = self.get_covered_tiles(center_lat_long, zoom, &screen_size);
        for vector_id in covered_tiles.iter() {
            if !self
                .vector_tile_manager_impl
                .lock()
                .unwrap()
                .is_tile_loaded(vector_name.clone(), vector_id)
            {
                let url = self.get_tile_request_url(&vector_id, url_template.as_ref());
                println!("-- covered {:?} @ {}", vector_id, url);

                let responder = self.vector_tile_manager_impl.clone();
                self.resource.get(&url, responder);
            }
        }
    }

    pub fn get_center_point_tile_xy(&self, center_lat_long: (f32, f32), zoom: f32) -> (f32, f32) {
        Tranformation::latlong_to_tile_coord(
            center_lat_long.0,
            center_lat_long.1,
            zoom.floor() as u32,
        )
    }

    // Stateless function - no threading concerns
    // TODO: to support rotation later
    pub fn get_covered_tiles(
        &self,
        center_lat_long: &(f32, f32),
        _zoom: f32,
        screen_size: &(u32, u32),
    ) -> Vec<VectorTileID> {
        let (tile_x, tile_y) = self.get_center_point_tile_xy(center_lat_long.to_owned(), self.zoom);

        let delta_tile_x = screen_size.0 as f32 / 2.0 / (TILE_SIZE as f32);
        let delta_tile_y = screen_size.1 as f32 / 2.0 / (TILE_SIZE as f32);
        let (min_tile_x_u32, min_tile_y_u32) = (
            (tile_x - delta_tile_x).floor() as u32,
            (tile_y - delta_tile_y).floor() as u32,
        );
        let (max_tile_x_u32, max_tile_y_u32) = (
            (tile_x + delta_tile_x).floor() as u32,
            (tile_y + delta_tile_y).floor() as u32,
        );

        let mut covered_tiles = vec![];

        for y in min_tile_y_u32..=max_tile_y_u32 {
            for x in min_tile_x_u32..=max_tile_x_u32 {
                covered_tiles.push(VectorTileID {
                    x,
                    y,
                    z: self.zoom.floor() as u32,
                });
            }
        }
        covered_tiles
    }

    fn get_tile_request_url(&self, tile_id: &VectorTileID, url_template: &str) -> String {
        let url = url_template.to_owned();
        url.replace("{x}", tile_id.x.to_string().as_ref())
            .replace("{y}", tile_id.y.to_string().as_ref())
            .replace("{z}", tile_id.z.to_string().as_ref())
    }

    pub fn add_vector_tile_observer(
        &mut self,
        vector_tile_obs: Threadable<dyn VectorTileObserver>,
    ) {
        self.vector_tile_manager_impl
            .lock()
            .unwrap()
            .add_vector_tile_observer(vector_tile_obs);
    }
}

//

struct VectorTileManagerImpl {
    loaded_tiles: HashMap<String, HashMap<VectorTileID, Arc<VectorTileModel>>>,
    painter_observer: Option<Threadable<dyn VectorTileObserver>>,
}

impl VectorTileManagerImpl {
    fn new() -> VectorTileManagerImpl {
        let mut loaded_tiles = HashMap::new();
        loaded_tiles.insert("COMPOSITE".to_string(), HashMap::new());
        loaded_tiles.insert("BUILDINGS".to_string(), HashMap::new());
        loaded_tiles.insert("INCIDENTS".to_string(), HashMap::new());
        loaded_tiles.insert("POI".to_string(), HashMap::new());
        VectorTileManagerImpl {
            loaded_tiles,
            painter_observer: None,
        }
    }

    fn is_tile_loaded(&self, vector_name: String, vector_tile_id: &VectorTileID) -> bool {
        self.loaded_tiles[&vector_name].contains_key(vector_tile_id)
    }

    fn get_tile_id_from_url(&self, url: &str) -> VectorTileID {
        let tokens: Vec<&str> = url.rsplit("/").collect();
        let (x, y, z) = (
            tokens[2].parse::<u32>().unwrap(),
            tokens[1].parse::<u32>().unwrap(),
            tokens[3].parse::<u32>().unwrap(),
        );
        VectorTileID { x, y, z }
    }

    pub fn add_vector_tile_observer(
        &mut self,
        vector_tile_obs: Threadable<dyn VectorTileObserver>,
    ) {
        self.painter_observer = Some(vector_tile_obs);
    }

    fn get_vector_tile_name_from_url(&self, url: String) -> String {
        // TODO: simple assumption: url is ..../COMPOSITE?v=4
        let segs: Vec<&str> = url.rsplit("/").collect();
        let with_ver = segs[0].to_string();
        let tokens: Vec<&str> = with_ver.split_terminator('?').collect();
        tokens[0].to_string()
    }
}

impl TaskResponder for VectorTileManagerImpl {
    fn on_task_success(&mut self, url: String, data: Option<Vec<u8>>) {
        println!("Yikes: VectorTile Load Succeeded from {}", &url);

        match data {
            Some(bytes) => {
                let mut tile_pbf = Pbf::new(bytes);
                let mut orig_parsed_tile = VectorTileModel::parse(&mut tile_pbf);
                orig_parsed_tile.normalize_coords();
                let parsed_tile = Arc::new(orig_parsed_tile);

                let vector_tile_id = self.get_tile_id_from_url(url.as_ref());
                println!(" -- Parsed VectorTile: {:?}", &vector_tile_id);

                let vector_name = self.get_vector_tile_name_from_url(url.clone());
                let named_loaded_tiles = self.loaded_tiles.get_mut(&vector_name).unwrap();
                named_loaded_tiles.insert(vector_tile_id.clone(), parsed_tile.clone());

                let tile_name = self.get_vector_tile_name_from_url(url);
                if self.painter_observer.is_some() {
                    self.painter_observer
                        .as_ref()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .on_vector_tile_loaded(tile_name, vector_tile_id, parsed_tile.clone());
                }
            }
            None => {
                println!("Error: empty VectorTile loaded");
            }
        }
    }

    fn on_task_failure(&self, map_error: MapError) {
        println!("Error: VectorTile Load Failed {}", map_error);
    }
}
