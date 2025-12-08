use crate::GeoNum;
use core::any::type_name;
use geo_types::*;
use std::convert::TryFrom;

use super::{
    MonotoneChainLineString, MonotoneChainMultiLineString, MonotoneChainMultiPolygon,
    MonotoneChainPolygon,
};
use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};

/// An enum representing any possible geometry type backed by [`MonotoneChain`](`crate::MonotoneChain`).
///
///  # Example
///
/// ```
/// use std::convert::TryFrom;
/// use geo::{LineString, line_string, MonotoneChainGeometry, Geometry};
///
/// let ls: LineString<f64> = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
/// let pe: Geometry = ls.into();
/// let lg: MonotoneChainGeometry<f64> = (&pe).try_into().expect("failed to convert");
/// ```
pub enum MonotoneChainGeometry<'a, T: GeoNum> {
    LineString(MonotoneChainLineString<'a, T>),
    MultiLineString(MonotoneChainMultiLineString<'a, T>),
    Polygon(MonotoneChainPolygon<'a, T>),
    MultiPolygon(MonotoneChainMultiPolygon<'a, T>),
}

impl<'a, T: GeoNum> TryFrom<&'a Geometry<T>> for MonotoneChainGeometry<'a, T> {
    type Error = Error;

    fn try_from(geometry: &'a Geometry<T>) -> Result<Self, Self::Error> {
        match geometry {
            Geometry::LineString(g) => Ok(Self::LineString((g).into())),
            Geometry::MultiLineString(g) => Ok(Self::MultiLineString((g).into())),
            Geometry::Polygon(g) => Ok(Self::Polygon((g).into())),
            Geometry::MultiPolygon(g) => Ok(Self::MultiPolygon((g).into())),
            other => Err(Error::MismatchedGeometry {
                expected: "LineString, MultiLineString, Polygon, MultiPolygon",
                found: inner_type_name(other),
            }),
        }
    }
}

/// Duplicated from geo-types
fn inner_type_name<T>(geometry: &Geometry<T>) -> &'static str
where
    T: CoordNum,
{
    match *geometry {
        Geometry::Point(_) => type_name::<Point<T>>(),
        Geometry::Line(_) => type_name::<Line<T>>(),
        Geometry::LineString(_) => type_name::<LineString<T>>(),
        Geometry::Polygon(_) => type_name::<Polygon<T>>(),
        Geometry::MultiPoint(_) => type_name::<MultiPoint<T>>(),
        Geometry::MultiLineString(_) => type_name::<MultiLineString<T>>(),
        Geometry::MultiPolygon(_) => type_name::<MultiPolygon<T>>(),
        Geometry::GeometryCollection(_) => type_name::<GeometryCollection<T>>(),
        Geometry::Rect(_) => type_name::<Rect<T>>(),
        Geometry::Triangle(_) => type_name::<Triangle<T>>(),
    }
}
