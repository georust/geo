#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expected a {expected}, but found a {found}")]
    MismatchedGeometry {
        expected: &'static str,
        found: &'static str,
    },
}

#[cfg(test)]
mod test {
    use crate::{Geometry, Point, Rect};
    use alloc::string::ToString;
    use core::convert::TryFrom;

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
            "Expected a geo_types::geometry::point::Point, but found a geo_types::geometry::rect::Rect"
        );
    }
}
