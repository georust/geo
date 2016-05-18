pub use ::Geometry;

use std::ops::Neg;
use num::Num;

pub trait ToGeo<T: Num + Copy>
{
    fn to_geo(&self) -> Geometry<T>;
}
