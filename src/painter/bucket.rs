use crate::mapbox::common::types::{Threadable, ThreadableNew};
use crate::mapbox::vector_tile_id::VectorTileID;
use crate::mapbox::vector_tile_model::VectorTileModel;

use gfx;

use std::collections::HashMap;
use std::sync::Arc;

pub type ResourceType = gfx_device_gl::Resources;
pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    constant Transform {
        screen_ratio: f32 = "u_ScreenRatio",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub type TransformType = Transform;
pub type VertexType = Vertex;

pub struct RenderableItem {
    pub geometry_type: u32,
    pub data: pipe::Data<ResourceType>,
    pub slice: gfx::Slice<ResourceType>,
}

pub struct Bucket {
    // TODO: make below non pub
    pub vector_tiles_map: Threadable<HashMap<String, HashMap<VectorTileID, Arc<VectorTileModel>>>>,
    // TODO: renderable data
    pub renderable_vector_tiles_map:
        Threadable<HashMap<String, HashMap<VectorTileID, Threadable<Vec<RenderableItem>>>>>,
}

impl Bucket {
    pub fn new() -> Self {
        let vector_tiles_map = ThreadableNew(HashMap::new());
        let renderable_vector_tiles_map = ThreadableNew(HashMap::new());
        for vector_tile_name in ["COMPOSITE", "BUILDINGS"].iter() {
            vector_tiles_map
                .lock()
                .unwrap()
                .insert(vector_tile_name.to_string(), HashMap::new());
            renderable_vector_tiles_map
                .lock()
                .unwrap()
                .insert(vector_tile_name.to_string(), HashMap::new());
        }
        Bucket {
            vector_tiles_map,
            renderable_vector_tiles_map,
        }
    }

    pub fn contains(&self, vector_name: &str, vector_tile_id: VectorTileID) -> bool {
        let vector_tiles_map = self.vector_tiles_map.lock().unwrap();
        if !vector_tiles_map.contains_key(vector_name) {
            return false;
        }

        let vector_tile_data = vector_tiles_map.get(vector_name).unwrap();
        vector_tile_data.contains_key(&vector_tile_id)
    }

    pub fn is_vector_tile_renderable(
        &self,
        vector_name: &str,
        vector_tile_id: VectorTileID,
    ) -> bool {
        let renderable_vector_tiles_map = self.renderable_vector_tiles_map.lock().unwrap();
        if !renderable_vector_tiles_map.contains_key(vector_name) {
            return false;
        }

        let renderable_vector_tile_data = renderable_vector_tiles_map.get(vector_name).unwrap();
        renderable_vector_tile_data.contains_key(&vector_tile_id)
    }

    pub fn remove(&mut self, vector_name: &str, vector_tile_id: VectorTileID) {
        let mut vector_tiles_map = self.vector_tiles_map.lock().unwrap();
        if vector_tiles_map.contains_key(vector_name) {
            let vector_tile_data = vector_tiles_map.get_mut(vector_name).unwrap();
            vector_tile_data.remove(&vector_tile_id);
        }

        let mut renderable_vector_tiles_map = self.renderable_vector_tiles_map.lock().unwrap();
        if renderable_vector_tiles_map.contains_key(vector_name) {
            let renderable_vector_tile_data =
                renderable_vector_tiles_map.get_mut(vector_name).unwrap();
            renderable_vector_tile_data.remove(&vector_tile_id);
        }
    }

    pub fn add_vector_tile_data(
        &mut self,
        vector_name: &str,
        vector_tile_id: VectorTileID,
        parsed_vector_tile: Arc<VectorTileModel>,
    ) {
        let mut vector_tiles_map = self.vector_tiles_map.lock().unwrap();
        let vector_tile_data = vector_tiles_map.get_mut(vector_name);
        if vector_tile_data.is_some() {
            vector_tile_data
                .unwrap()
                .insert(vector_tile_id, parsed_vector_tile.clone());
        }
    }

    pub fn add_renderable_item(
        &mut self,
        vector_tile_name: String,
        vector_tile_id: VectorTileID,
        renderable_item: RenderableItem,
    ) {
        let mut renderable_vector_tiles_map = self.renderable_vector_tiles_map.lock().unwrap();
        let renderable_vector_tile_data = renderable_vector_tiles_map
            .get_mut(&vector_tile_name)
            .unwrap();
        if !renderable_vector_tile_data.contains_key(&vector_tile_id) {
            renderable_vector_tile_data.insert(vector_tile_id, ThreadableNew(vec![]));
        }
        let renderable_vector_tile_data_array = renderable_vector_tile_data
            .get_mut(&vector_tile_id)
            .unwrap();

        renderable_vector_tile_data_array
            .lock()
            .unwrap()
            .push(renderable_item);
    }

    pub fn get_renderable_items(
        &self,
        vector_tile_name: String,
        vector_tile_id: VectorTileID,
    ) -> Threadable<Vec<RenderableItem>> {
        let renderable_vector_tiles_map = self.renderable_vector_tiles_map.lock().unwrap();
        let renderable_vector_tile_data =
            renderable_vector_tiles_map.get(&vector_tile_name).unwrap();

        renderable_vector_tile_data
            .get(&vector_tile_id)
            .unwrap()
            .clone()
    }
}
