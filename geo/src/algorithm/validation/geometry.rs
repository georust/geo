use super::{ProblemReport, Validation};
use crate::{GeoFloat, Geometry};

impl<F: GeoFloat> Validation for Geometry<F> {
    fn is_valid(&self) -> bool {
        match self {
            Geometry::Point(e) => e.is_valid(),
            Geometry::Line(e) => e.is_valid(),
            Geometry::Rect(e) => e.is_valid(),
            Geometry::Triangle(e) => e.is_valid(),
            Geometry::LineString(e) => e.is_valid(),
            Geometry::Polygon(e) => e.is_valid(),
            Geometry::MultiPoint(e) => e.is_valid(),
            Geometry::MultiLineString(e) => e.is_valid(),
            Geometry::MultiPolygon(e) => e.is_valid(),
            Geometry::GeometryCollection(e) => e.is_valid(),
        }
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        match self {
            Geometry::Point(e) => e.explain_invalidity(),
            Geometry::Line(e) => e.explain_invalidity(),
            Geometry::Rect(e) => e.explain_invalidity(),
            Geometry::Triangle(e) => e.explain_invalidity(),
            Geometry::LineString(e) => e.explain_invalidity(),
            Geometry::Polygon(e) => e.explain_invalidity(),
            Geometry::MultiPoint(e) => e.explain_invalidity(),
            Geometry::MultiLineString(e) => e.explain_invalidity(),
            Geometry::MultiPolygon(e) => e.explain_invalidity(),
            Geometry::GeometryCollection(e) => e.explain_invalidity(),
        }
    }
}
