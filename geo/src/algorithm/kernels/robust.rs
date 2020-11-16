use super::{Kernel, Orientation};
use crate::Coordinate;

/// Robust kernel that uses [fast robust
/// predicates](//www.cs.cmu.edu/~quake/robust.html) to
/// provide robust floating point predicates. Should only be
/// used with types that can _always_ be casted to `f64`
/// _without loss in precision_.
#[derive(Default)]
pub struct RobustKernel;

use num_traits::{Float, NumCast};
impl<T: Float> Kernel<T> for RobustKernel {
    fn orient2d(p: Coordinate<T>, q: Coordinate<T>, r: Coordinate<T>) -> Orientation {
        use robust::{orient2d, Coord};

        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from(p.x).unwrap(),
                y: <f64 as NumCast>::from(p.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(q.x).unwrap(),
                y: <f64 as NumCast>::from(q.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(r.x).unwrap(),
                y: <f64 as NumCast>::from(r.y).unwrap(),
            },
        );

        if orientation < 0. {
            Orientation::Clockwise
        } else if orientation > 0. {
            Orientation::CounterClockwise
        } else {
            Orientation::Collinear
        }
    }
}
