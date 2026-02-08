mod utils;
mod geojson;

use crate::geojson::{Geometry, Feature, FeatureCollection};
use std::io;
use std::{convert::TryFrom, io::{Cursor, ErrorKind}};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;
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

fn parse_bbox(rdr: &mut Cursor<&[u8]>) -> Result<BBox, Error> {
    let min = rdr.read_f64::<LittleEndian>()?;
    let max = rdr.read_f64::<LittleEndian>()?;
    return Ok(BBox{min, max});
}

fn parse_fileheader(mut rdr: &mut Cursor<&[u8]>) -> Result<ShapeFileHeader, Error> {
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
    let x_bbox = parse_bbox(&mut rdr)?;
    let y_bbox = parse_bbox(&mut rdr)?;
    let z_bbox = parse_bbox(&mut rdr)?;
    let m_bbox = parse_bbox(&mut rdr)?;
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

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

fn parse_point(rdr: &mut Cursor<&[u8]>) -> Result<Point, std::io::Error> {
    let x = rdr.read_f64::<LittleEndian>()?;
    let y = rdr.read_f64::<LittleEndian>()?;
    Ok(Point{x, y})
}
struct Polygon {
    x_bbox: BBox,
    y_bbox: BBox,
    parts: Vec<i32>,
    points: Vec<Point>,
}

fn parse_polygon(mut rdr: &mut Cursor<&[u8]>) -> Result<Polygon, std::io::Error> {
    let x_min = rdr.read_f64::<LittleEndian>()?;
    let y_min = rdr.read_f64::<LittleEndian>()?;
    let x_max = rdr.read_f64::<LittleEndian>()?;
    let y_max = rdr.read_f64::<LittleEndian>()?;
    let num_parts = rdr.read_i32::<LittleEndian>()?;
    let num_points = rdr.read_i32::<LittleEndian>()?;
    let mut parts = Vec::new();
    for _ in 0..num_parts {
        let part = rdr.read_i32::<LittleEndian>()?;
        parts.push(part);
    };
    let mut points = Vec::new();
    for _ in 0..num_points {
        let point = parse_point(&mut rdr)?;
        points.push(point);
    };
    return Ok(Polygon{
        x_bbox:BBox { min: x_min, max: x_max },
        y_bbox:BBox { min: y_min, max: y_max },
        parts,
        points,
    })
}

const WORD_PER_BYTES: i32 = 2;

#[wasm_bindgen]
pub fn parse_shp(data: &[u8]) -> Result<FeatureCollection, JsValue> {
    let mut rdr = Cursor::new(data);

    let header = parse_fileheader(&mut rdr).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let all_record_length = header.file_length * WORD_PER_BYTES - 25;
    let mut i = 0;
    let mut features = Vec::new();
    while i < all_record_length {
        let record_header = match read_record_header(&mut rdr) {
            Ok(None) => break,
            Ok(Some(v)) => v,
            Err(e) => return Err(JsValue::from_str(&Error::ParseError(e).to_string())),
        };
        i += (record_header.length + 4) * WORD_PER_BYTES;
        let raw_shape_type = rdr.read_i32::<LittleEndian>().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let shape_typ = ShapeType::try_from(raw_shape_type).map_err(|e| JsValue::from_str(&e.to_string()))?;
        if header.shape_type != shape_typ {
            return Err(JsValue::from_str("not match"))
        }
        let geom = match shape_typ {
            ShapeType::POINT => {
                let record_content = parse_point(&mut rdr).map_err(|e| JsValue::from_str(&Error::ParseError(e).to_string()))?;
                Geometry::Point([record_content.x, record_content.y])
            },
            ShapeType::POLYGON => {
                let content = parse_polygon(&mut rdr).map_err(|e| JsValue::from_str(&Error::ParseError(e).to_string()))?;
                let mut parts = content.parts.clone();
                parts.push(content.points.len() as i32);
                let polygons = parts.windows(2).into_iter().map(|w| {
                    let start = w[0] as usize;
                    let end = w[1] as usize;
                    content.points[start..end].iter().map(|p| [p.x, p.y]).collect()
                }).collect();
                Geometry::Polygon(polygons)
            },
            _ => return Err(JsValue::from_str(&Error::UnsupportedShapeType(header.shape_type).to_string())),
        };
        let feature = Feature::new(geom);
        features.push(feature);
    }
    let collection = FeatureCollection::new(features);
    Ok(collection)
}