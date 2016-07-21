pub use ::Geometry;

use num::Float;

pub trait ToRadians {
    fn to_radians(self) -> Self;
}

impl ToRadians for f32 {
    fn to_radians(self) -> f32 {
        self.to_radians()
    }
}

impl ToRadians for f64 {
    fn to_radians(self) -> f64 {
        self.to_radians()
    }
}

pub trait ToGeo<T: Float>
{
    fn to_geo(&self) -> Geometry<T>;
}
