pub use ::Geometry;


pub trait ToGeo {
    fn to_geo(&self) -> Geometry;
}
