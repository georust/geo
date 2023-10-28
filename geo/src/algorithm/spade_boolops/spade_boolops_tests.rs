use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::HasDimensions;

use super::*;
use geo_types::*;
use wkt::TryFromWkt;

// helper
//
pub fn multipolygon_from<F: SpadeTriangulationFloat>(v: Vec<Polygon<F>>) -> MultiPolygon<F> {
    MultiPolygon::new(v)
}

pub fn is_multipolygon_nonempty<F: SpadeTriangulationFloat>(multipolygon: &MultiPolygon<F>) {
    let condition_true =
        !multipolygon.is_empty() && !multipolygon.iter().any(|poly| poly.is_empty());
    assert!(
        condition_true,
        "polygon was empty even though non-empty was expected"
    );
}

pub fn is_multipolygon_empty<F: SpadeTriangulationFloat>(multipolygon: &MultiPolygon<F>) {
    let condition_true = multipolygon.is_empty() && multipolygon.iter().all(|poly| poly.is_empty());
    assert!(
        condition_true,
        "polygon was non-empty even though empty was expected"
    );
}

pub fn has_num_holes<F: SpadeTriangulationFloat>(
    multipolygon: &MultiPolygon<F>,
    expected_num_holes: usize,
) {
    let false_num_holes = multipolygon
        .iter()
        .map(|poly| poly.interiors().len())
        .find(|&num_holes| num_holes != expected_num_holes);
    assert!(false_num_holes.is_none(), "A polygon had not the expected number of holes ({expected_num_holes}), but {} holes instead\n\n{multipolygon:?}", false_num_holes.unwrap());
}

pub fn has_num_vertices<F: SpadeTriangulationFloat>(
    multipolygon: &MultiPolygon<F>,
    expected_num_vertices: usize,
) {
    let false_num_vertices = multipolygon
        .iter()
        .map(|poly| poly.exterior().coords().count())
        .find(|&num_vertices| num_vertices != expected_num_vertices);
    assert!(false_num_vertices.is_none(), "A polygon had not the expected number of vertices ({expected_num_vertices}), but {} vertices instead", false_num_vertices.unwrap());
}

pub fn has_num_polygons<F: SpadeTriangulationFloat>(
    multipolygon: &MultiPolygon<F>,
    expected_num_polys: usize,
) {
    assert_eq!(
        multipolygon.0.len(),
        expected_num_polys,
        "A multipolygon had not the expected number of polygons ({expected_num_polys}), but {} polygons instead",
        multipolygon.0.len()
    );
}

fn load_wkt(data_str: &str) -> Result<Vec<Polygon<f32>>, String> {
    let GeometryCollection(data) =
        GeometryCollection::<f32>::try_from_wkt_str(data_str).map_err(|e| format!("{e:?}"))?;
    let data = data
        .into_iter()
        .filter_map(|g| g.try_into().ok())
        .collect::<Vec<_>>();
    Ok(data)
}

#[test]
fn basic_intersection_compiles() {
    let zero = Coord::zero();
    let one = Coord { x: 1.0, y: 1.0 };
    let rect1 = Rect::new(zero, one * 2.0);
    let rect2 = Rect::new(one, one * 3.0);

    SpadeBoolops::intersection(&rect1.to_polygon(), &rect2.to_polygon()).unwrap();
}

#[test]
fn star_intersects_self_properly() {
    _ = pretty_env_logger::try_init();
    let data = include_str!("./data/star.wkt");
    let data = load_wkt(data).unwrap();
    let poly1 = &data[0];

    let intersection = Polygon::intersection(poly1, poly1).unwrap();

    is_multipolygon_nonempty(&intersection);
    has_num_polygons(&intersection, 1);
    has_num_holes(&intersection, 0);
    has_num_vertices(&intersection, 23);
}

#[test]
fn star_shape_slightly_offset_difference() {
    _ = pretty_env_logger::try_init();
    let data = include_str!("./data/star.wkt");
    let data = load_wkt(data).unwrap();
    let poly1 = &data[0];

    let mut poly2 = poly1.clone();
    poly2.exterior_mut(|ext| {
        ext.coords_mut().skip(1).take(1).for_each(|coord| {
            *coord = *coord + Coord { x: 0.1, y: 0.1 };
        });
    });

    let difference = Polygon::difference(poly1, &poly2).unwrap();

    is_multipolygon_nonempty(&difference);
    has_num_polygons(&difference, 1);
    has_num_holes(&difference, 0);
    has_num_vertices(&difference, 4);
}
