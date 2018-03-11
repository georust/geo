pub use Geometry;

use types::CoordinateType;

pub trait ToGeo<T: CoordinateType>
{
    fn to_geo(&self) -> Geometry<T>;
}
