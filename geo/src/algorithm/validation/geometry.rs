use super::Validation;
use super::{
    InvalidGeometryCollection, InvalidLine, InvalidLineString, InvalidMultiLineString,
    InvalidMultiPoint, InvalidMultiPolygon, InvalidPoint, InvalidPolygon, InvalidRect,
    InvalidTriangle,
};
use crate::{GeoFloat, Geometry};

use crate::geometry_cow::GeometryCow;
use std::fmt;

/// A [`Geometry`] is valid if its inner variant is valid.
/// e.g. `Geometry::Polygon(polygon)` is valid if and only if `polygon` is valid.
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidGeometry {
    InvalidPoint(InvalidPoint),
    InvalidLine(InvalidLine),
    InvalidLineString(InvalidLineString),
    InvalidPolygon(InvalidPolygon),
    InvalidMultiPoint(InvalidMultiPoint),
    InvalidMultiLineString(InvalidMultiLineString),
    InvalidMultiPolygon(InvalidMultiPolygon),
    InvalidGeometryCollection(InvalidGeometryCollection),
    InvalidRect(InvalidRect),
    InvalidTriangle(InvalidTriangle),
}

impl fmt::Display for InvalidGeometry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidGeometry::InvalidPoint(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidLine(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidLineString(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidPolygon(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidMultiPoint(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidMultiLineString(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidMultiPolygon(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidGeometryCollection(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidRect(err) => write!(f, "{}", err),
            InvalidGeometry::InvalidTriangle(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for InvalidGeometry {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InvalidGeometry::InvalidPoint(err) => Some(err),
            InvalidGeometry::InvalidLine(err) => Some(err),
            InvalidGeometry::InvalidLineString(err) => Some(err),
            InvalidGeometry::InvalidPolygon(err) => Some(err),
            InvalidGeometry::InvalidMultiPoint(err) => Some(err),
            InvalidGeometry::InvalidMultiLineString(err) => Some(err),
            InvalidGeometry::InvalidMultiPolygon(err) => Some(err),
            InvalidGeometry::InvalidGeometryCollection(err) => Some(err),
            InvalidGeometry::InvalidRect(err) => Some(err),
            InvalidGeometry::InvalidTriangle(err) => Some(err),
        }
    }
}

impl<F: GeoFloat> Validation for Geometry<F> {
    type Error = InvalidGeometry;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        match self {
            Geometry::Point(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidPoint(err))
            }))?,
            Geometry::Line(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidLine(err))
            }))?,
            Geometry::LineString(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidLineString(err))
            }))?,
            Geometry::Polygon(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidPolygon(err))
            }))?,
            Geometry::MultiPoint(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiPoint(err))
            }))?,
            Geometry::MultiLineString(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiLineString(err))
            }))?,
            Geometry::MultiPolygon(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiPolygon(err))
            }))?,
            Geometry::GeometryCollection(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidGeometryCollection(err))
            }))?,
            Geometry::Rect(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidRect(err))
            }))?,
            Geometry::Triangle(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidTriangle(err))
            }))?,
        }
        Ok(())
    }
}

impl<F: GeoFloat> Validation for GeometryCow<'_, F> {
    type Error = InvalidGeometry;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        match self {
            GeometryCow::Point(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidPoint(err))
            }))?,
            GeometryCow::Line(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidLine(err))
            }))?,
            GeometryCow::LineString(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidLineString(err))
            }))?,
            GeometryCow::Polygon(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidPolygon(err))
            }))?,
            GeometryCow::MultiPoint(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiPoint(err))
            }))?,
            GeometryCow::MultiLineString(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiLineString(err))
            }))?,
            GeometryCow::MultiPolygon(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidMultiPolygon(err))
            }))?,
            GeometryCow::GeometryCollection(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidGeometryCollection(err))
            }))?,
            GeometryCow::Rect(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidRect(err))
            }))?,
            GeometryCow::Triangle(g) => g.visit_validation(Box::new(|err| {
                handle_validation_error(InvalidGeometry::InvalidTriangle(err))
            }))?,
        }
        Ok(())
    }
}
