use crate::config;
use crate::mapbox::config::TILE_SIZE;

pub struct TileGrid {}

impl TileGrid {
    pub fn gen_tile_grid_screen_coords(
        center_tile_xy: (f32, f32),
        target_tile_xy: (f32, f32),
    ) -> (f32, f32, f32, f32) {
        // (top_left_x, top_left_y, bottom_right_x, bottom_right_y)
        let (normalized_center_tile_x, normalized_center_tile_y) = (
            center_tile_xy.0 - center_tile_xy.0.floor(),
            center_tile_xy.1 - center_tile_xy.1.floor(),
        );

        let (normalized_tile_w_screen_ratio, normalized_tile_h_screen_ratio) =
            Self::get_normalized_tile_dim_screen_ratio();

        let (screen_center_delta_x, screen_center_delta_y) = (
            normalized_center_tile_x * normalized_tile_w_screen_ratio,
            normalized_center_tile_y * normalized_tile_h_screen_ratio,
        );

        let (
            center_tile_top_left_x,
            center_tile_top_left_y,
            center_tile_bottom_right_x,
            center_tile_bottom_right_y,
        ) = (
            -screen_center_delta_x,
            screen_center_delta_y,
            normalized_tile_w_screen_ratio - screen_center_delta_x,
            screen_center_delta_y - normalized_tile_h_screen_ratio,
        );

        // Tile Coord Delta
        let (tile_coord_delta_x, tile_coord_delta_y) = (
            target_tile_xy.0.floor() - center_tile_xy.0.floor(),
            target_tile_xy.1.floor() - center_tile_xy.1.floor(),
        );

        let (tile_screen_delta_x, tile_screen_delta_y) = (
            tile_coord_delta_x * normalized_tile_w_screen_ratio,
            tile_coord_delta_y * normalized_tile_h_screen_ratio,
        );

        (
            center_tile_top_left_x + tile_screen_delta_x,
            center_tile_top_left_y - tile_screen_delta_y,
            center_tile_bottom_right_x + tile_screen_delta_x,
            center_tile_bottom_right_y - tile_screen_delta_y,
        )
    }

    pub fn get_normalized_tile_dim_screen_ratio() -> (f32, f32) {
        (
            TILE_SIZE as f32 / (config::GL_VIEWPORT_WIDTH / 2) as f32,
            TILE_SIZE as f32 / (config::GL_VIEWPORT_HEIGHT / 2) as f32,
        )
    }
}
