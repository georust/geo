use crate::geometry::*;
use crate::GeoFloat;

#[cfg(test)]
mod tests;

pub enum InvalidGeometry {
    InvalidCoordinate,
}

pub trait Validation {
    fn check_validation(&self) -> Result<(), InvalidGeometry>;
    fn is_valid(&self) -> bool {
        self.check_validation().is_ok()
    }
}

impl<T: GeoFloat> Validation for Geometry<T> {
    crate::geometry_delegate_impl! {
        fn check_validation(&self) -> Result<(), InvalidGeometry>;
    }
}

impl<T: GeoFloat> Validation for Point<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for Point")
    }
}

impl<T: GeoFloat> Validation for LineString<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for LineString")
    }
}

impl<T: GeoFloat> Validation for Polygon<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for Polygon")
    }
}

impl<T: GeoFloat> Validation for MultiPoint<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for MultiPoint")
    }
}

impl<T: GeoFloat> Validation for MultiLineString<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for MultiLineString")
    }
}

impl<T: GeoFloat> Validation for MultiPolygon<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for MultiPolygon")
    }
}

impl<T: GeoFloat> Validation for GeometryCollection<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        todo!("Implement validation for GeometryCollection")
    }
}

impl<T: GeoFloat> Validation for Line<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        // REVIEW:
        LineString(vec![self.start, self.end]).check_validation()
    }
}

impl<T: GeoFloat> Validation for Rect<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        // REVIEW:
        self.to_polygon().check_validation()
    }
}

impl<T: GeoFloat> Validation for Triangle<T> {
    fn check_validation(&self) -> Result<(), InvalidGeometry> {
        // REVIEW:
        self.to_polygon().check_validation()
    }
}
