use super::{
    utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, ProblemReport,
    Validation,
};
use crate::{CoordFloat, Triangle};

/// As stated in geo-types/src/geometry/triangles.rs,
/// "the three vertices must not be collinear and they must be distinct"
impl<T> Validation for Triangle<T>
where
    T: CoordFloat,
{
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.0)
            || utils::check_coord_is_not_finite(&self.1)
            || utils::check_coord_is_not_finite(&self.2)
        {
            return false;
        }

        if self.0 == self.1 || self.1 == self.2 || self.2 == self.0 {
            return false;
        }

        if utils::robust_check_points_are_collinear::<T>(&self.0, &self.1, &self.2) {
            return false;
        }
        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.0) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.1) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(1)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.2) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(2)),
            ));
        }

        // We wont check if the points are collinear if they are identical
        let mut identical = false;

        if self.0 == self.1 || self.0 == self.2 {
            reason.push(ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Triangle(CoordinatePosition(0)),
            ));
            identical = true;
        }

        if self.1 == self.2 {
            reason.push(ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Triangle(CoordinatePosition(1)),
            ));
            identical = true;
        }

        if !identical && utils::robust_check_points_are_collinear::<T>(&self.0, &self.1, &self.2) {
            reason.push(ProblemAtPosition(
                Problem::CollinearCoords,
                ProblemPosition::Triangle(CoordinatePosition(-1)),
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
    use crate::Triangle;

    #[test]
    fn test_triangle_valid() {
        let t = Triangle((0., 0.).into(), (0., 1.).into(), (0.5, 2.).into());
        assert!(t.is_valid());
        assert!(t.explain_invalidity().is_none());
    }

    #[test]
    fn test_triangle_invalid_same_points() {
        let t = Triangle((0., 0.).into(), (0., 1.).into(), (0., 1.).into());
        assert!(!t.is_valid());
        assert_eq!(
            t.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Triangle(CoordinatePosition(1)),
            )]))
        );
    }

    #[test]
    fn test_triangle_invalid_points_collinear() {
        let t = Triangle((0., 0.).into(), (1., 1.).into(), (2., 2.).into());
        assert!(!t.is_valid());
        assert_eq!(
            t.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::CollinearCoords,
                ProblemPosition::Triangle(CoordinatePosition(-1)),
            )]))
        );
    }

    // #[test]
    // fn test_triangle_invalid_points_collinear2() {
    //     let t = Triangle((0, 0).into(), (1, 1).into(), (2, 2).into());
    //     assert!(!t.is_valid());
    //     assert_eq!(
    //         t.explain_invalidity(),
    //         Some(vec![ProblemAtPosition(
    //             Problem::CollinearCoords,
    //             ProblemPosition::Triangle(CoordinatePosition(-1)),
    //         )])
    //     );
    // }
}
