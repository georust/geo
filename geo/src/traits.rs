pub use crate::Geometry;

use crate::CoordinateType;

#[deprecated(
    note = "Will be removed in an upcoming version. Switch to std::convert::Into<Geo> or std::convert::TryInto<Geo>."
)]
pub trait ToGeo<T: CoordinateType> {
    fn to_geo(&self) -> Geometry<T>;
}
