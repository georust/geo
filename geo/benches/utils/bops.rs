use geo_types::{MultiPolygon, Polygon, Coordinate, LineString};

pub fn convert_poly(poly: &Polygon<f64>) -> gt_prev::Polygon<f64> {
    let ext: Vec<_> = poly
        .exterior()
        .0
        .iter()
        .map(|c| gt_prev::Coordinate { x: c.x, y: c.y })
        .collect();
    gt_prev::Polygon::new(gt_prev::LineString(ext), vec![])
}

pub fn convert_mpoly(mpoly: &MultiPolygon<f64>) -> gt_prev::MultiPolygon<f64> {
    mpoly.0.iter().map(convert_poly).collect()
}

pub fn convert_back_poly(poly: &gt_prev::Polygon<f64>) -> Polygon<f64> {
    let ext: Vec<_> = poly
        .exterior()
        .0
        .iter()
        .map(|c| Coordinate { x: c.x, y: c.y })
        .collect();
    Polygon::new(LineString(ext), vec![])
}

pub fn convert_back_mpoly(mpoly: &gt_prev::MultiPolygon<f64>) -> MultiPolygon<f64> {
    mpoly.0.iter().map(convert_back_poly).collect()
}
