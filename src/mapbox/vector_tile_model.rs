// https://docs.mapbox.com/vector-tiles/specification/
//
// [Geometry Encoding]
// To encode geographic information into a vector tile a tool must convert geographic coordinates,
// such as latitude and longitude, into vector tile grid coordinates. Vector tiles hold no concept
// of geographic information. They encode points, lines, and polygons as x/y pairs relative to the
// top left of the grid in a right-down manner.
//
// [Attribute Encoding]
// Attributes are encoded in a series of tags that exist within a feature in the vector that have
// integer values that reference keys and values designating the original key:value pairs from the
// geometry. For large geometry, this removes redundancy for attributes that have the same keys and
// similar values.
//
// [Winding Order]
// Exterior rings must be oriented clockwise and interior rings must be oriented counter-clockwise
// (when viewed in screen coordinates).
//
// A layer MUST contain an extent that describes the width and height of the tile in integer
// coordinates. The geometries within the Vector Tile MAY extend past the bounds of the tile's area
// as defined by the extent. Geometries that extend past the tile's area as defined by extent are
// often used as a buffer for rendering features that overlap multiple adjacent tiles.
//
// https://github.com/mapbox/vector-tile-spec/tree/master/2.0
//
// https://github.com/mapbox/vector-tile-spec/blob/master/2.1/vector_tile.proto

use super::utils::pbf::Pbf;

use std::collections::HashMap;

#[derive(Debug)]
pub struct VectorTileModel {
    pub layers: Vec<VectorTileLayer>,
}

impl VectorTileModel {
    pub fn parse(data: &mut Pbf) -> Self {
        let mut vector_tile_model = VectorTileModel { layers: vec![] };

        while data.next() {
            if data.tag == 3 {
                let mut layer_msg = data.message();
                let layer = VectorTileLayer::parse(&mut layer_msg);
                vector_tile_model.layers.push(layer);
            } else {
                data.skip();
            }
        }
        vector_tile_model
    }

    pub fn normalize_coords(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.normalize_coords();
        }
    }
}

#[derive(Debug)]
pub struct VectorTileLayer {
    pub name: String,
    pub features: Vec<VectorTileFeature>,
    pub keys: Vec<String>,
    pub values: Vec<VectorTileValue>,
    pub extent: u32,
}

impl VectorTileLayer {
    pub fn parse(data: &mut Pbf) -> Self {
        let mut layer = VectorTileLayer {
            name: String::new(),
            features: vec![],
            keys: vec![],
            values: vec![],
            extent: 4096,
        };

        while data.next() {
            match data.tag {
                1 => {
                    // name
                    let name = data.string();
                    layer.name = name;
                }
                2 => {
                    // feature
                    let mut msg = data.message();
                    let feature = VectorTileFeature::parse(&mut msg);
                    layer.features.push(feature);
                }
                3 => {
                    // keys
                    let key = data.string();
                    layer.keys.push(key);
                }
                4 => {
                    // values
                    let mut msg = data.message();
                    let v = VectorTileValue::parse(&mut msg);
                    layer.values.push(v);
                }
                5 => {
                    // extent
                    let v = data.varint32();
                    layer.extent = v;
                }
                _ => {
                    data.skip();
                }
            }
        }

        layer
    }

    pub fn normalize_coords(&mut self) {
        let extent = self.extent;
        for feature in self.features.iter_mut() {
            feature.normalize_coords(extent);
        }
    }
}

#[derive(Debug)]
pub enum VectorTileValue {
    None,
    StringVal(String),
    Float32Val(f32),
    Float64Val(f64),
    Int64Val(i64),
    UInt64Val(u64),
    SInt64Val(i64),
    BoolVal(bool),
}

impl VectorTileValue {
    pub fn parse(data: &mut Pbf) -> Self {
        let mut val = VectorTileValue::None;
        while data.next() {
            match data.tag {
                1 =>
                // string_value
                {
                    val = VectorTileValue::StringVal(data.string());
                }
                2 =>
                // float_value
                {
                    val = VectorTileValue::Float32Val(data.fixed32());
                }
                3 =>
                // double_value
                {
                    val = VectorTileValue::Float64Val(data.fixed64());
                }
                4 =>
                // int_value
                {
                    val = VectorTileValue::Int64Val(data.svarint64());
                }
                5 =>
                // uint_value
                {
                    val = VectorTileValue::UInt64Val(data.varint64());
                }
                6 =>
                // sint_value
                {
                    val = VectorTileValue::SInt64Val(data.svarint64());
                }
                7 =>
                // bool_value
                {
                    val = VectorTileValue::BoolVal(data.boolean());
                }
                _ => {
                    data.skip();
                }
            }
        } // while
        val
    }
}

//
#[derive(Debug)]
pub struct VectorTileFeature {
    pub id: u64,
    pub r#type: u32,
    pub tags: HashMap<u32, u32>,
    pub geometry: Vec<VectorTileGeometry>,
}

impl VectorTileFeature {
    pub fn parse(data: &mut Pbf) -> Self {
        let mut vector_tile_feature = VectorTileFeature {
            id: 0,
            r#type: 0,
            tags: HashMap::new(),
            geometry: vec![],
        };

        while data.next() {
            match data.tag {
                1 => {
                    // id
                    let v = data.varint64();
                    vector_tile_feature.id = v;
                }
                2 => {
                    // tags
                    let mut tag_pbf = data.message();
                    vector_tile_feature.parse_tags(&mut tag_pbf);
                }
                3 => {
                    // type
                    let t = data.varint32();
                    vector_tile_feature.r#type = t;
                }
                4 => {
                    // geometry
                    let mut geometry_pbf = data.message();
                    let geometry = VectorTileGeometry::parse(&mut geometry_pbf);
                    vector_tile_feature.geometry.push(geometry);
                }
                _ => {
                    data.skip();
                }
            }
        }

        vector_tile_feature
    }

    fn parse_tags(&mut self, data: &mut Pbf) {
        while data.has_next() {
            let tag_key = data.varint32();

            //if (layer.keys.size() <= tag_key) {
            //    throw std::runtime_error("feature referenced out of range key");
            //}

            if !data.has_next() {
                panic!("uneven number of feature tag ids");
            }

            let tag_val = data.varint32();
            //if (layer.values.size() <= tag_val) {
            //    panic!("feature referenced out of range value");
            //}

            self.tags.insert(tag_key, tag_val);
        } // while
    }

    pub fn normalize_coords(&mut self, extent: u32) {
        for geom in self.geometry.iter_mut() {
            geom.normalize_coords(extent as f32);
        }
    }
}

#[derive(Debug)]
pub struct VectorTileGeometry {
    // Using f32 here so that we can also support normalized coords
    pub geom_set: Vec<Vec<(f32, f32)>>,
}

impl VectorTileGeometry {
    pub fn parse(data: &mut Pbf) -> Self {
        let mut vector_tile_geometry = VectorTileGeometry { geom_set: vec![] };

        let mut cmd: u8 = 1;
        let mut length: u32 = 0;
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;

        let mut current_points = vec![];
        while data.has_next() {
            if length == 0 {
                let cmd_length = data.varint32();
                cmd = (cmd_length & 0x7) as u8;
                length = cmd_length >> 3;
            }

            length -= 1;

            if cmd == 1 || cmd == 2 {
                if cmd == 1 && !current_points.is_empty() {
                    // moveTo - push current set and start a new set for next set.
                    vector_tile_geometry.geom_set.push(current_points.clone());
                    current_points.clear();
                }

                // get absolute coords from relative coords
                // TODO: handle buffer area (for coords > 4096)
                x += data.svarint32() as f32;
                y += data.svarint32() as f32;

                current_points.push((x, y));
            } else if cmd == 7 {
                // closePolygon
                if !current_points.is_empty() {
                    let first_copied = current_points[0].clone();
                    current_points.push(first_copied);
                    vector_tile_geometry.geom_set.push(current_points.clone());
                }
            } else {
                panic!("unknown command");
            }
        }

        // points geometry doesn't close with closePolygon
        if !current_points.is_empty() {
            vector_tile_geometry.geom_set.push(current_points.clone());
        }

        vector_tile_geometry
    }

    // Normalize all coords within the scope of (0.0, 1.0) to get ready
    // for rendering which uses normalized coords.
    pub fn normalize_coords(&mut self, extent: f32) {
        for point in self.geom_set.iter_mut() {
            for d in point.iter_mut() {
                d.0 /= extent;
                d.1 /= extent;
            }
        }
    }
}
