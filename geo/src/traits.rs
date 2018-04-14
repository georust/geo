pub use Geometry;

use ::CoordinateType;

pub trait ToGeo<T: CoordinateType>
{
    fn to_geo(&self) -> Geometry<T>;
}
