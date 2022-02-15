#[derive(PartialEq, Eq, Hash, Debug, Copy)]
pub struct VectorTileID {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Clone for VectorTileID {
    fn clone(&self) -> Self {
        *self
    }
}
