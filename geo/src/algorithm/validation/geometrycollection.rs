use crate::{GeometryPosition, ProblemAtPosition, ProblemPosition, ProblemReport, Valid};
use geo::{GeoFloat, GeometryCollection};

/// GeometryCollection is valid if all its elements are valid
impl<F: GeoFloat> Valid for GeometryCollection<F> {
    fn is_valid(&self) -> bool {
        for geometry in self.0.iter() {
            if !geometry.is_valid() {
                return false;
            }
        }
        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        // Loop over all the geometries, collect the reasons of invalidity
        // and change the ProblemPosition to reflect the GeometryCollection
        for (i, geometry) in self.0.iter().enumerate() {
            let temp_reason = geometry.explain_invalidity();
            if let Some(temp_reason) = temp_reason {
                for ProblemAtPosition(problem, position) in temp_reason.0 {
                    reason.push(ProblemAtPosition(
                        problem,
                        ProblemPosition::GeometryCollection(
                            GeometryPosition(i),
                            Box::new(position),
                        ),
                    ));
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
    use crate::{
        CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition,
        ProblemReport, Valid,
    };
    use geo::{Coord, Geometry, GeometryCollection, LineString, Point};
    use geos::Geom;

    #[test]
    fn test_geometrycollection_contain_invalid_element() {
        let gc = GeometryCollection(vec![
            Geometry::Point(Point::new(0., 0.)),
            Geometry::LineString(LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
            ])),
            Geometry::LineString(LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 0., y: 0. },
            ])),
        ]);
        assert!(!gc.is_valid());
        assert_eq!(
            gc.explain_invalidity(),
            Some(ProblemReport(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::GeometryCollection(
                    GeometryPosition(2),
                    Box::new(ProblemPosition::LineString(CoordinatePosition(0)))
                )
            )]))
        );

        let geoms =
            gc.0.iter()
                .map(|geometry| match geometry {
                    Geometry::Point(pt) => {
                        let geos_point: geos::Geometry = pt.try_into().unwrap();
                        geos_point
                    }
                    Geometry::LineString(ls) => {
                        let geos_linestring: geos::Geometry = ls.try_into().unwrap();
                        geos_linestring
                    }
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
        let geometrycollection_geos: geos::Geometry =
            geos::Geometry::create_geometry_collection(geoms).unwrap();
        assert_eq!(gc.is_valid(), geometrycollection_geos.is_valid());
    }
}
