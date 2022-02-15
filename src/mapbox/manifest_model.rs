// Ref: https://docs.mapbox.com/mapbox-gl-js/style-spec/#root-sources
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ManifestType {
    // TODO: https://crates.io/crates/strum
    Vector,
    Raster,
}

#[derive(Debug)]
pub struct ManifestModel {
    pub r#type: ManifestType,
    pub name: String,
    pub scheme: String,
    pub minzoom: i64,
    pub maxzoom: i64,
    pub tilezooms: Vec<u64>,
    pub tilejson: String,
    pub prefetchable: bool,
    pub priority: i64,
    pub bounds: Vec<f64>,
    pub center: Vec<f64>,
    pub tiles: Vec<String>,
}

impl ManifestModel {
    pub fn new() -> ManifestModel {
        ManifestModel {
            r#type: ManifestType::Vector,
            name: String::new(),
            scheme: String::new(),
            minzoom: 0,
            maxzoom: 0,
            tilezooms: vec![],
            tilejson: String::new(),
            prefetchable: false,
            priority: 0,
            bounds: vec![],
            center: vec![],
            tiles: vec![],
        }
    }

    pub fn parse(type_: &ManifestType, json_value: Value) -> ManifestModel {
        ManifestModel {
            r#type: type_.clone(),
            name: String::from(json_value["name"].as_str().unwrap_or_default()),
            scheme: String::from(json_value["scheme"].as_str().unwrap_or_default()),
            minzoom: json_value["minzoom"].as_i64().unwrap_or_default(),
            maxzoom: json_value["maxzoom"].as_i64().unwrap_or_default(),
            tilezooms: json_value["tilezooms"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_u64().unwrap_or_default())
                .collect(),
            tilejson: json_value["tilejson"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            prefetchable: json_value["prefetchable"].as_bool().unwrap_or_default(),
            priority: json_value["priority"].as_i64().unwrap_or_default(),
            bounds: json_value["bounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap_or_default())
                .collect(),
            center: json_value["center"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap_or_default())
                .collect(),
            tiles: json_value["tiles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap_or_default().to_owned())
                .collect(),
        }
    }
}
