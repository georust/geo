use super::{GeometryPosition, ProblemAtPosition, ProblemPosition, ProblemReport, Validation};
use crate::{GeoFloat, MultiLineString};

/// MultiLineString is valid if all its LineStrings are valid.
impl<F: GeoFloat> Validation for MultiLineString<F> {
    fn is_valid(&self) -> bool {
        for line in &self.0 {
            if !line.is_valid() {
                return false;
            }
        }
        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        for (j, line) in self.0.iter().enumerate() {
            let temp_reason = line.explain_invalidity();
            if let Some(temp_reason) = temp_reason {
                for ProblemAtPosition(problem, position) in temp_reason.0 {
                    match position {
                        ProblemPosition::LineString(coord_pos) => {
                            reason.push(ProblemAtPosition(
                                problem,
                                ProblemPosition::MultiLineString(GeometryPosition(j), coord_pos),
                            ));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(ProblemReport(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition,
        ProblemReport, Validation,
    };
    use crate::{Coord, LineString, MultiLineString};
    use geos::Geom;

    #[test]
    fn test_multilinestring_valid() {
        let mls = MultiLineString(vec![
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            LineString(vec![Coord { x: 3., y: 1. }, Coord { x: 4., y: 1. }]),
        ]);
        assert!(mls.is_valid());
        assert!(mls.explain_invalidity().is_none());

        // Test that the linestring has the same validity status than its GEOS equivalent
        let multilinestring_geos: geos::Geometry = (&mls).try_into().unwrap();
        assert_eq!(mls.is_valid(), multilinestring_geos.is_valid());
    }

    #[test]
    fn test_multilinestring_invalid_too_few_points_with_duplicate() {
        // The second LineString (at position 1) of this MultiLineString
        // is not valid because it has only one (deduplicated) point
        let mls = MultiLineString(vec![
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 0., y: 0. }]),
        ]);
        assert!(!mls.is_valid());
        assert_eq!(
            mls.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::MultiLineString(GeometryPosition(1), CoordinatePosition(0))
            )]))
        );

        // Test that the linestring has the same validity status than its GEOS equivalent
        let multilinestring_geos: geos::Geometry = (&mls).try_into().unwrap();
        assert_eq!(mls.is_valid(), multilinestring_geos.is_valid());
    }
}
