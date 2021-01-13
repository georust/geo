pub use crate::Geometry;

use crate::CoordNum;

#[deprecated(
    note = "Will be removed in an upcoming version. Switch to std::convert::Into<Geo> or std::convert::TryInto<Geo>."
)]
pub trait ToGeo<T: CoordNum> {
    fn to_geo(&self) -> Geometry<T>;
}
