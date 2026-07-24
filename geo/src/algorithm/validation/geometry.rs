use super::Validation;
use super::{
    InvalidGeometryCollection, InvalidLine, InvalidLineString, InvalidMultiLineString,
    InvalidMultiPoint, InvalidMultiPolygon, InvalidPoint, InvalidPolygon, InvalidRect,
    InvalidTriangle,
};
use crate::{GeoFloat, Geometry};

use crate::geometry_cow::GeometryCow;

/// A [`Geometry`] is valid if its inner variant is valid.
/// e.g. `Geometry::Polygon(polygon)` is valid if and only if `polygon` is valid.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidGeometry {
    #[error(transparent)]
    InvalidPoint(#[from] InvalidPoint),
    #[error(transparent)]
    InvalidLine(#[from] InvalidLine),
    #[error(transparent)]
    InvalidLineString(#[from] InvalidLineString),
    #[error(transparent)]
    InvalidPolygon(#[from] InvalidPolygon),
    #[error(transparent)]
    InvalidMultiPoint(#[from] InvalidMultiPoint),
    #[error(transparent)]
    InvalidMultiLineString(#[from] InvalidMultiLineString),
    #[error(transparent)]
    InvalidMultiPolygon(#[from] InvalidMultiPolygon),
    #[error(transparent)]
    InvalidGeometryCollection(#[from] InvalidGeometryCollection),
    #[error(transparent)]
    InvalidRect(#[from] InvalidRect),
    #[error(transparent)]
    InvalidTriangle(#[from] InvalidTriangle),
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
