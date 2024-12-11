use super::{GeometryIndex, InvalidPolygon, Validation};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::{GeoFloat, MultiPolygon, Relate};

use std::fmt;

/// A [`MultiPolygon`] is valid if:
/// - [x] all its polygons are valid,
/// - [x] elements do not overlaps (i.e. their interiors must not intersect)
/// - [x] elements touch only at points
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidMultiPolygon {
    /// For a [`MultiPolygon`] to be valid, each member [`Polygon`](crate::Polygon) must be valid.
    InvalidPolygon(GeometryIndex, InvalidPolygon),
    /// No [`Polygon`](crate::Polygon) in a valid [`MultiPolygon`] may overlap (2-dimensional intersection)
    ElementsOverlaps(GeometryIndex, GeometryIndex),
    /// No [`Polygon`](crate::Polygon) in a valid [`MultiPolygon`] may touch on a line (1-dimensional intersection)
    ElementsTouchOnALine(GeometryIndex, GeometryIndex),
}

impl fmt::Display for InvalidMultiPolygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidMultiPolygon::InvalidPolygon(idx, err) => {
                write!(f, "polygon at index {} is invalid: {}", idx.0, err)
            }
            InvalidMultiPolygon::ElementsOverlaps(idx1, idx2) => {
                write!(f, "polygons at indices {} and {} overlap", idx1.0, idx2.0)
            }
            InvalidMultiPolygon::ElementsTouchOnALine(idx1, idx2) => {
                write!(
                    f,
                    "polygons at indices {} and {} touch on a line",
                    idx1.0, idx2.0
                )
            }
        }
    }
}

impl std::error::Error for InvalidMultiPolygon {}

impl<F: GeoFloat> Validation for MultiPolygon<F> {
    type Error = InvalidMultiPolygon;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        for (i, polygon) in self.0.iter().enumerate() {
            polygon.visit_validation(Box::new(&mut |invalid_polygon| {
                handle_validation_error(InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(i),
                    invalid_polygon,
                ))
            }))?;

            // Special case for MultiPolygon: elements must not overlap and must touch only at points
            for (j, pol2) in self.0.iter().enumerate().skip(i + 1) {
                let im = polygon.relate(pol2);
                if im.get(CoordPos::Inside, CoordPos::Inside) == Dimensions::TwoDimensional {
                    let err =
                        InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(i), GeometryIndex(j));
                    handle_validation_error(err)?;
                }
                if im.get(CoordPos::OnBoundary, CoordPos::OnBoundary) == Dimensions::OneDimensional
                {
                    let err = InvalidMultiPolygon::ElementsTouchOnALine(
                        GeometryIndex(i),
                        GeometryIndex(j),
                    );
                    handle_validation_error(err)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::assert_validation_errors;
    use super::*;
    use crate::algorithm::validation::RingRole;
    use crate::wkt;

    #[test]
    fn test_multipolygon_invalid() {
        // The following multipolygon contains two invalid polygon
        // and it is invalid itself because the two polygons of the multipolygon are not disjoint
        // (here they are identical)
        let multi_polygon = wkt!(
            MULTIPOLYGON (
                (
                    (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                    (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
                ),
                (
                    (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                    (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
                )
            )
        );
        assert_validation_errors!(
            &multi_polygon,
            vec![
                InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(0),
                    InvalidPolygon::InteriorRingNotContainedInExteriorRing(RingRole::Interior(0))
                ),
                InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(0), GeometryIndex(1)),
                InvalidMultiPolygon::ElementsTouchOnALine(GeometryIndex(0), GeometryIndex(1)),
                InvalidMultiPolygon::InvalidPolygon(
                    GeometryIndex(1),
                    InvalidPolygon::InteriorRingNotContainedInExteriorRing(RingRole::Interior(0))
                ),
            ]
        );
    }
}
