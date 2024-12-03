use crate::{ProblemReport, Valid};
use geo::{GeoFloat, Point};

/// In PostGIS, Point don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl<T> Valid for Point<T>
where
    T: GeoFloat,
{
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        self.0.explain_invalidity()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Valid};
    use geo::Point;
    use geos::Geom;

    #[test]
    fn test_point_valid() {
        let p = Point::new(0., 0.);
        assert!(p.is_valid());
        assert!(p.explain_invalidity().is_none());

        // Test that the point has the same validity status than its GEOS equivalent
        let pt_geos: geos::Geometry = (&p).try_into().unwrap();
        assert_eq!(p.is_valid(), pt_geos.is_valid());
    }

    #[test]
    fn test_point_invalid() {
        let p = Point::new(f64::NAN, f64::NAN);
        assert!(!p.is_valid());
        assert_eq!(
            p.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point
            )]))
        );

        let pt_geos: geos::Geometry = (&p).try_into().unwrap();
        assert_eq!(p.is_valid(), pt_geos.is_valid());
    }
}
