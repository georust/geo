use crate::{
    utils, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Valid,
};
use geo::{GeoFloat, MultiPoint};

/// In PostGIS, MultiPoint don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl<F: GeoFloat> Valid for MultiPoint<F> {
    fn is_valid(&self) -> bool {
        for point in &self.0 {
            if !point.is_valid() {
                return false;
            }
        }
        true
    }

    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        for (i, point) in self.0.iter().enumerate() {
            if utils::check_coord_is_not_finite(&point.0) {
                reason.push(ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(i)),
                ));
            }
        }

        if reason.is_empty() {
            None
        } else {
            Some(ProblemReport(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Valid,
    };
    use geo::{MultiPoint, Point};
    use geos::Geom;

    #[test]
    fn test_multipoint_valid() {
        let mp = MultiPoint(vec![Point::new(0., 0.), Point::new(1., 1.)]);
        assert!(mp.is_valid());
        assert!(mp.explain_invalidity().is_none());

        // This multipoint is invalid according to this crate but valid according to GEOS
        let multipoint_geos: geos::Geometry = (&mp).try_into().unwrap();
        assert_eq!(mp.is_valid(), multipoint_geos.is_valid());
    }

    #[test]
    fn test_multipoint_invalid() {
        let mp = MultiPoint(vec![
            Point::new(0., f64::INFINITY),
            Point::new(f64::NAN, 1.),
        ]);
        assert!(!mp.is_valid());
        assert_eq!(
            mp.explain_invalidity(),
            Some(ProblemReport(vec![
                ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(0))
                ),
                ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(1))
                )
            ]))
        );

        // Test that this multipoint has the same validity status than its GEOS equivalent
        let multipoint_geos: geos::Geometry = (&mp).try_into().unwrap();
        assert_eq!(mp.is_valid(), multipoint_geos.is_valid());
    }
}
