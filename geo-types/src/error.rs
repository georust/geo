use std::fmt;

#[derive(Debug)]
pub enum Error {
    MismatchedGeometry {
        expected: &'static str,
        found: &'static str,
    },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MismatchedGeometry { expected, found } => {
                write!(f, "Expected a {}, but found a {}", expected, found)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Geometry, Point, Rect};
    use std::convert::TryFrom;

    #[test]
    fn error_output() {
        let point = Point::new(1.0, 2.0);
        let point_geometry = Geometry::from(point);

        let rect = Rect::new(Point::new(1.0, 2.0), Point::new(3.0, 4.0));
        let rect_geometry = Geometry::from(rect);

        Point::try_from(point_geometry).expect("failed to unwrap inner enum Point");

        let failure = Point::try_from(rect_geometry).unwrap_err();
        assert_eq!(
            failure.to_string(),
            "Expected a geo_types::point::PointTZM<f64, geo_types::novalue::NoValue, geo_types::novalue::NoValue>, but found a geo_types::rect::RectTZM<f64, geo_types::novalue::NoValue, geo_types::novalue::NoValue>"
        );
    }
}
