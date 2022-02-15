// Ref: https://docs.mapbox.com/mapbox-gl-js/style-spec/
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct StyleModel {
    pub name: String,
    pub min_zoom: i64,
    pub max_zoom: i64,
    pub sprite_url: String,
    pub glyph_template_url: String,
    pub sources: Map<String, Value>,
    pub layers: Vec<Value>,
}

impl StyleModel {
    pub fn new() -> StyleModel {
        StyleModel {
            name: String::new(),
            min_zoom: 0,
            max_zoom: 0,
            sprite_url: String::new(),
            glyph_template_url: String::new(),
            sources: Map::new(),
            layers: vec![],
        }
    }

    pub fn parse(json_value: Value) -> StyleModel {
        let layers = json_value["layers"].as_array().unwrap();
        let sources = json_value["sources"].as_object().unwrap();
        StyleModel {
            name: String::from(json_value["name"].as_str().unwrap_or_default()),
            min_zoom: json_value["minzoom"].as_i64().unwrap_or_default(),
            max_zoom: json_value["maxzoom"].as_i64().unwrap_or_default(),
            sprite_url: String::from(json_value["sprite"].as_str().unwrap_or_default()),
            glyph_template_url: String::from(json_value["glyphs"].as_str().unwrap_or_default()),
            layers: layers.to_owned(),
            sources: sources.to_owned(),
        }
    }
}
