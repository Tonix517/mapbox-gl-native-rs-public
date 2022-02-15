use super::vector_tile_id::VectorTileID;
use super::vector_tile_model::VectorTileModel;

use std::sync::Arc;

pub trait VectorTileObserver: Send {
    fn on_vector_tile_loaded(
        &mut self,
        name: String,
        vector_tile_id: VectorTileID,
        parsed_vector_tile: Arc<VectorTileModel>,
    );
}
