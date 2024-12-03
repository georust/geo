use crate::{
    utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Valid,
};
use geo::{GeoFloat, Rect};

impl<F: GeoFloat> Valid for Rect<F> {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.min())
            || utils::check_coord_is_not_finite(&self.max())
        {
            return false;
        }
        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.min()) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Rect(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.max()) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Rect(CoordinatePosition(1)),
            ));
        }

        if reason.is_empty() {
            None
        } else {
            Some(ProblemReport(reason))
        }
    }
}
