use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::HasDimensions;

use geo_types::*;
use wkt::TryFromWkt;

pub fn multipolygon_from<F: SpadeTriangulationFloat>(v: Vec<Polygon<F>>) -> MultiPolygon<F> {
    MultiPolygon::new(v)
}

pub fn empty_multipoly<F: SpadeTriangulationFloat>() -> MultiPolygon<F> {
    MultiPolygon::new(vec![])
}

pub fn empty_poly<F: SpadeTriangulationFloat>() -> Polygon<F> {
    Polygon::new(LineString::new(vec![]), vec![])
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
    mut expected_num_holes: Vec<usize>,
) {
    let calc_num_hole = |poly: &Polygon<F>| poly.interiors().len();
    multipolygon.iter().for_each(|poly| {
        let num_holes = calc_num_hole(poly);
        if let Some(pos) = expected_num_holes.iter().position(|&n| n == num_holes) {
            expected_num_holes.remove(pos);
        }
    });
    let has_right_num_holes = expected_num_holes.is_empty();
    let num_holes = multipolygon
        .iter()
        .map(|p| calc_num_hole(p))
        .collect::<Vec<_>>();
    assert!(has_right_num_holes, "A polygon had not the expected number of holes ({expected_num_holes:?}), but {num_holes:?} holes instead");
}

/// This exists soley for getting notified (or warned if you will) about non-trivial changes to the
/// algorithm and not to assert a desired amount of vertices.
///
/// If, for example, a change to the algorithm would simplify the resulting geometry, then these
/// assertions would notify the developer. They could then check if the changes make sense.
///
/// In the end this is just a way to prevent errors slipping through the hands of the devs
pub fn has_num_vertices<F: SpadeTriangulationFloat>(
    multipolygon: &MultiPolygon<F>,
    mut expected_num_vertices: Vec<usize>,
) {
    let calc_num_vertices = |poly: &Polygon<F>| {
        poly.exterior().coords().count()
            + poly
                .interiors()
                .iter()
                .map(|i| i.coords().count())
                .sum::<usize>()
    };
    multipolygon.iter().for_each(|poly| {
        let num_verts = calc_num_vertices(poly);
        if let Some(pos) = expected_num_vertices.iter().position(|&n| n == num_verts) {
            expected_num_vertices.remove(pos);
        }
    });
    let has_right_num_vertices = expected_num_vertices.is_empty();
    let num_vertices = multipolygon
        .iter()
        .map(|p| calc_num_vertices(p))
        .collect::<Vec<_>>();
    assert!(has_right_num_vertices, "A polygon had not the expected number of vertices ({expected_num_vertices:?}), but {num_vertices:?} vertices instead");
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

pub fn load_wkt(data_str: &str) -> Result<Vec<Polygon<f32>>, String> {
    let GeometryCollection(data) =
        GeometryCollection::<f32>::try_from_wkt_str(data_str).map_err(|e| format!("{e:?}"))?;
    let data = data
        .into_iter()
        .filter_map(|g| g.try_into().ok())
        .collect::<Vec<_>>();
    Ok(data)
}
