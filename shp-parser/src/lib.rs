mod utils;

use std::io;
use std::{convert::TryFrom, io::{Cursor, ErrorKind}};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use thiserror::Error;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, shp-parser!");
}

#[derive(Error, Debug)]
enum Error {
    #[error("parse error: {0}")]
    ParseError(#[from] io::Error),
    #[error("invalid shape_type: {0}")]
    InvalidShapeType(i32),
    #[error("unsupported shape_type: {0:?}")]
    UnsupportedShapeType(ShapeType),
    #[error("unknown error")]
    Unknown,
}

#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(i32)]
enum ShapeType {
    NULL = 0,
    POINT = 1,
	POLYLINE    = 3,
	POLYGON     = 5,
	MULTIPOINT  = 8,
	POINTZ      = 11,
	POLYLINEZ   = 13,
	POLYGONZ    = 15,
	MULTIPOINTZ = 18,
	POINTM      = 21,
	POLYLINEM   = 23,
	POLYGONM    = 25,
	MULTIPOINTM = 28,
	MULTIPATCH  = 31
}

struct BBox {
    min: f64,
    max: f64,
}

struct ShapeFileHeader {
    magic_number: i32,
    file_length: i32,
    version: i32,
    shape_type: ShapeType,
    x_bbox: BBox,
    y_bbox: BBox,
    z_bbox: BBox,
    m_bbox: BBox,
}

fn parse_fileheader(rdr: &mut Cursor<&[u8]>) -> Result<ShapeFileHeader, Error> {
    let magic_number = rdr.read_i32::<BigEndian>()?;
    for _ in 0..5 {
        rdr.read_i32::<BigEndian>()?;
    }
    let file_length = rdr.read_i32::<BigEndian>()?;
    let version = rdr.read_i32::<LittleEndian>()?;
    if magic_number != 9994 && version != 1000 {
        alert("invalid shape file");
    }
    let shape_type = ShapeType::try_from(rdr.read_i32::<LittleEndian>()?).map_err(|i| Error::InvalidShapeType(i.number))?;
    let x_bbox_min = rdr.read_f64::<LittleEndian>()?;
    let x_bbox_max = rdr.read_f64::<LittleEndian>()?;
    let x_bbox = BBox{min: x_bbox_min, max: x_bbox_max};
    let y_bbox_min = rdr.read_f64::<LittleEndian>()?;
    let y_bbox_max = rdr.read_f64::<LittleEndian>()?;
    let y_bbox = BBox{ min: y_bbox_min, max: y_bbox_max};
    let z_bbox_min = rdr.read_f64::<LittleEndian>()?;
    let z_bbox_max = rdr.read_f64::<LittleEndian>()?;
    let z_bbox = BBox{ min: z_bbox_min, max: z_bbox_max};
    let m_bbox_min = rdr.read_f64::<LittleEndian>()?;
    let m_bbox_max = rdr.read_f64::<LittleEndian>()?;
    let m_bbox = BBox{min: m_bbox_min, max: m_bbox_max};
    Ok(
        ShapeFileHeader { magic_number, file_length, version, shape_type, x_bbox, y_bbox, z_bbox, m_bbox }
    )
}

struct ContentHeader {
    number: i32,
    length: i32,
}

fn read_record_header(rdr: &mut Cursor<&[u8]>) -> Result<Option<ContentHeader>, std::io::Error> {
    let number = match rdr.read_i32::<BigEndian>() {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
            return Ok(None);
        },
        Err(e) => return Err(e),
    };
    let length = rdr.read_i32::<BigEndian>()?;
    return Ok(Some(ContentHeader { number, length }))
}

struct Point {
    x: f64,
    y: f64,
}

fn parse_point(rdr: &mut Cursor<&[u8]>) -> Result<Point, std::io::Error> {
    let shape_typ = rdr.read_i32::<LittleEndian>()?;
    let x = rdr.read_f64::<LittleEndian>()?;
    let y = rdr.read_f64::<LittleEndian>()?;
    Ok(Point{x, y})
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "coordinates")]
pub enum Geometry {
    Point([f64; 2]),
    LineString(Vec<[f64; 2]>),
    Polygon(Vec<Vec<[f64; 2]>>),
    MultiPoint(Vec<[f64; 2]>),
    MultiLineString(Vec<Vec<[f64; 2]>>),
    MultiPolygon(Vec<Vec<Vec<[f64; 2]>>>),
}

#[derive(Debug, Serialize)]
pub struct Feature {
    #[serde(rename = "type")]
    feature_type: &'static str, // 常に "Feature"
    pub geometry: Geometry,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    collection_type: &'static str, // "FeatureCollection"
    pub features: Vec<Feature>,
}

#[wasm_bindgen]
pub fn parse_shp(data: &[u8]) -> Result<JsValue, JsValue> {
    let mut rdr = Cursor::new(data);

    let header = parse_fileheader(&mut rdr).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let all_record_length = header.file_length * 2 - 25;
    let mut i = 0;
    let mut features = Vec::new();
    while i < all_record_length {
        let record_header = match read_record_header(&mut rdr) {
            Ok(None) => break,
            Ok(Some(v)) => v,
            Err(e) => return Err(JsValue::from_str(&Error::ParseError(e).to_string())),
        };
        i += (record_header.length + 4) * 2;
        let record_content = match header.shape_type {
            ShapeType::POINT => parse_point(&mut rdr).map_err(|e| Error::ParseError(e)),
            _ => return Err(JsValue::from_str(&Error::UnsupportedShapeType(header.shape_type).to_string())),
        };
        let geom = Geometry::Point([record_content.as_ref().unwrap().x, record_content.as_ref().unwrap().y]);
        let feature = Feature{feature_type: "Point", geometry: geom, properties: HashMap::new()};
        features.push(feature);
    }
    let collection = FeatureCollection {
        collection_type: "FeatureCollection",
        features: features
    };
    Ok(serde_wasm_bindgen::to_value(&collection)?)
}