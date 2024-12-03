use super::{
    utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport,
    Validation,
};
use crate::{GeoFloat, Line};

impl<F: GeoFloat> Validation for Line<F> {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.start)
            || utils::check_coord_is_not_finite(&self.end)
        {
            return false;
        }

        if self.start == self.end {
            return false;
        }

        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.start) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Line(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.end) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Line(CoordinatePosition(1)),
            ));
        }

        if self.start == self.end {
            reason.push(ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Line(CoordinatePosition(0)),
            ));
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
    use super::super::{
        CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Validation,
    };
    use crate::Line;

    #[test]
    fn test_line_valid() {
        let l = Line::new((0., 0.), (1., 1.));
        assert!(l.is_valid());
        assert!(l.explain_invalidity().is_none());
    }

    #[test]
    fn test_line_invalid_not_finite_coords() {
        let l = Line::new((0., 0.), (f64::NEG_INFINITY, 0.));
        assert!(!l.is_valid());
        assert_eq!(
            l.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Line(CoordinatePosition(1)),
            )]))
        );
    }

    #[test]
    fn test_line_invalid_same_points() {
        let l = Line::new((0., 0.), (0., 0.));
        assert!(!l.is_valid());
        assert_eq!(
            l.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Line(CoordinatePosition(0)),
            )]))
        );
    }
}
