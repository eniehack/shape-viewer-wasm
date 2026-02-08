use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[serde(tag = "type", content = "coordinates")]
pub enum Geometry {
    Point([f64; 2]),
    LineString(Vec<[f64; 2]>),
    Polygon(Vec<Vec<[f64; 2]>>),
    MultiPoint(Vec<[f64; 2]>),
    MultiLineString(Vec<Vec<[f64; 2]>>),
    MultiPolygon(Vec<Vec<Vec<[f64; 2]>>>),
}

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Feature {
    #[serde(rename = "type")]
    #[tsify(type = "'Feature'")]
    typ: &'static str, // 常に "Feature"
    pub geometry: Geometry,
    pub properties: HashMap<String, String>,
}
impl Feature {
    pub fn new(geometry: Geometry) -> Self {
        Self { typ: "Feature", geometry, properties: HashMap::new() }
    }
}


#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    #[tsify(type = "'FeatureCollection'")]
    typ: &'static str, // "FeatureCollection"
    pub features: Vec<Feature>,
}

impl FeatureCollection {
    pub fn new(features: Vec<Feature>) -> Self {
        Self { typ: "FeatureCollection", features }
    }
}
