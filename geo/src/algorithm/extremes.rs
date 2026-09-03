use crate::CoordsIter;
use crate::{Coord, CoordNum};

/// Find the extreme coordinates and indices of a geometry.
///
/// # Examples
///
/// ```
/// use geo::extremes::Extremes;
/// use geo::polygon;
///
/// // a diamond shape
/// let polygon = polygon![
///     (x: 1.0, y: 0.0),
///     (x: 2.0, y: 1.0),
///     (x: 1.0, y: 2.0),
///     (x: 0.0, y: 1.0),
///     (x: 1.0, y: 0.0),
/// ];
///
/// let extremes = polygon.extremes().unwrap();
///
/// assert_eq!(extremes.y_max.index, 2);
/// assert_eq!(extremes.y_max.coord.x, 1.);
/// assert_eq!(extremes.y_max.coord.y, 2.);
/// ```
pub trait Extremes<'a, T: CoordNum> {
    fn extremes(&'a self) -> Option<Outcome<T>>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Extreme<T: CoordNum> {
    pub index: usize,
    pub coord: Coord<T>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Outcome<T: CoordNum> {
    pub x_min: Extreme<T>,
    pub y_min: Extreme<T>,
    pub x_max: Extreme<T>,
    pub y_max: Extreme<T>,
}

impl<'a, T, G> Extremes<'a, T> for G
where
    G: CoordsIter<Scalar = T>,
    T: CoordNum,
{
    fn extremes(&'a self) -> Option<Outcome<T>> {
        let mut iter = self.exterior_coords_iter().enumerate();

        let mut outcome = iter.next().map(|(index, coord)| Outcome {
            x_min: Extreme { index, coord },
            y_min: Extreme { index, coord },
            x_max: Extreme { index, coord },
            y_max: Extreme { index, coord },
        })?;

        for (index, coord) in iter {
            if coord.x < outcome.x_min.coord.x {
                outcome.x_min = Extreme { coord, index };
            }

            if coord.y < outcome.y_min.coord.y {
                outcome.y_min = Extreme { coord, index };
            }

            if coord.x > outcome.x_max.coord.x {
                outcome.x_max = Extreme { coord, index };
            }

            if coord.y > outcome.y_max.coord.y {
                outcome.y_max = Extreme { coord, index };
            }
        }

        Some(outcome)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{MultiPoint, coord, polygon};

    #[test]
    fn polygon() {
        // a diamond shape
        let polygon = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0),
        ];

        let actual = polygon.extremes();

        assert_eq!(
            Some(Outcome {
                x_min: Extreme {
                    index: 3,
                    coord: coord! { x: 0.0, y: 1.0 }
                },
                y_min: Extreme {
                    index: 0,
                    coord: coord! { x: 1.0, y: 0.0 }
                },
                x_max: Extreme {
                    index: 1,
                    coord: coord! { x: 2.0, y: 1.0 }
                },
                y_max: Extreme {
                    index: 2,
                    coord: coord! { x: 1.0, y: 2.0 }
                }
            }),
            actual
        );
    }

    #[test]
    fn empty() {
        let multi_point: MultiPoint<f32> = MultiPoint::empty();

        let actual = multi_point.extremes();

        assert!(actual.is_none());
    }
}

#[cfg(test)]
mod hegel_props {
    use super::Extremes;
    use crate::utils::hegel_gens::geometries;
    use crate::{BoundingRect, CoordsIter};

    // `extremes` reports "the extreme coordinates and indices of a geometry", so
    // each `Extreme` must name a coordinate that actually attains that bound,
    // and the index must be the first one that does — the implementation
    // compares strictly, keeping the earliest.
    #[hegel::test]
    fn each_extreme_is_the_first_coord_attaining_that_bound(tc: hegel::TestCase) {
        let geometry = tc.draw(geometries(1e12));
        let Some(outcome) = geometry.extremes() else {
            return;
        };
        let coords: Vec<_> = geometry.exterior_coords_iter().collect();
        for (extreme, key) in [
            (&outcome.x_min, 0),
            (&outcome.x_max, 0),
            (&outcome.y_min, 1),
            (&outcome.y_max, 1),
        ] {
            let component = |c: &crate::Coord<f64>| if key == 0 { c.x } else { c.y };
            assert_eq!(coords[extreme.index], extreme.coord);
            let first = coords
                .iter()
                .position(|c| component(c) == component(&extreme.coord))
                .expect("the extreme coord came from this iterator");
            assert_eq!(first, extreme.index);
        }
        for coord in &coords {
            assert!(coord.x >= outcome.x_min.coord.x && coord.x <= outcome.x_max.coord.x);
            assert!(coord.y >= outcome.y_min.coord.y && coord.y <= outcome.y_max.coord.y);
        }
    }

    // Both traits are blanket-implemented over `CoordsIter`, and both summarise
    // the same coordinate extremes, so the bounds have to agree.
    #[hegel::test]
    fn extremes_agree_with_the_bounding_rect(tc: hegel::TestCase) {
        let geometry = tc.draw(geometries(1e12));
        let (Some(outcome), Some(rect)) = (geometry.extremes(), geometry.bounding_rect()) else {
            assert!(geometry.extremes().is_none() && geometry.bounding_rect().is_none());
            return;
        };
        assert_eq!(outcome.x_min.coord.x, rect.min().x);
        assert_eq!(outcome.y_min.coord.y, rect.min().y);
        assert_eq!(outcome.x_max.coord.x, rect.max().x);
        assert_eq!(outcome.y_max.coord.y, rect.max().y);
    }
}
