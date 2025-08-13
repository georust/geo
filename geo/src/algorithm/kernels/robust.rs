use super::{CoordNum, Kernel, Orientation};
use geo_traits::CoordTrait;

use num_traits::{Float, NumCast};

/// Robust kernel that uses [fast robust
/// predicates](//www.cs.cmu.edu/~quake/robust.html) to
/// provide robust floating point predicates. Should only be
/// used with types that can _always_ be casted to `f64`
/// _without loss in precision_.
#[derive(Default, Debug)]
pub struct RobustKernel;

impl<P> Kernel<P> for RobustKernel
where
    P: CoordTrait,
    P::T: CoordNum + Float,
{
    fn orient2d(p: P, q: P, r: P) -> Orientation {
        use robust::{orient2d, Coord};

        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from(p.x()).unwrap(),
                y: <f64 as NumCast>::from(p.y()).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(q.x()).unwrap(),
                y: <f64 as NumCast>::from(q.y()).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(r.x()).unwrap(),
                y: <f64 as NumCast>::from(r.y()).unwrap(),
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
