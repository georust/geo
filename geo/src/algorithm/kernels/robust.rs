use super::Kernel;
use crate::Coordinate;
use crate::algorithm::winding_order::WindingOrder;
use std::marker::PhantomData;

#[derive(Default)]
pub struct RobustKernel<T>(PhantomData<T>);

use num_traits::{Float, NumCast};
impl<T: Float> Kernel for RobustKernel<T> {
    type Scalar = T;

    fn orient2d(
        p: Coordinate<Self::Scalar>,
        q: Coordinate<Self::Scalar>,
        r: Coordinate<Self::Scalar>,
    ) -> Option<WindingOrder> {
        use robust::{orient2d, Coord};

        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from( p.x ).unwrap(),
                y: <f64 as NumCast>::from( p.y ).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from( q.x ).unwrap(),
                y: <f64 as NumCast>::from( q.y ).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from( r.x ).unwrap(),
                y: <f64 as NumCast>::from( r.y ).unwrap(),
            },
        );

        if orientation < 0. {
            Some(WindingOrder::Clockwise)
        } else if orientation > 0. {
            Some(WindingOrder::CounterClockwise)
        } else {
            None
        }
    }
}
