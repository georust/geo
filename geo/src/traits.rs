pub use crate::Geometry;

use crate::CoordinateType;

pub trait ToGeo<T: CoordinateType> {
    fn to_geo(&self) -> Geometry<T>;
}
