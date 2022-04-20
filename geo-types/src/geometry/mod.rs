pub(crate) mod coordinate;
pub(crate) mod geometry_collection;
pub(crate) mod line;
pub(crate) mod line_string;
pub(crate) mod multi_line_string;
pub(crate) mod multi_point;
pub(crate) mod multi_polygon;
pub(crate) mod point;
pub(crate) mod polygon;
pub(crate) mod rect;
pub(crate) mod triangle;

// re-export all the geometry variants:
pub use coordinate::Coordinate;
pub use geometry_collection::GeometryCollection;
pub use line::Line;
pub use line_string::LineString;
pub use multi_line_string::MultiLineString;
pub use multi_point::MultiPoint;
pub use multi_polygon::MultiPolygon;
pub use point::Point;
pub use polygon::Polygon;
pub use rect::Rect;
pub use triangle::Triangle;

use crate::{CoordNum, Error};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use core::any::type_name;
use std::convert::TryFrom;

/// An enum representing any possible geometry type.
///
/// All geometry variants ([`Point`], [`LineString`], etc.) can be converted to a `Geometry` using
/// [`Into::into`]. Conversely, [`TryFrom::try_from`] can be used to convert a [`Geometry`]
/// _back_ to one of it's specific enum members.
///
/// # Example
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
///
/// let p = point!{ x: 1.0, y: 1.0 };
/// let pe: Geometry = p.into();
/// let pn = Point::try_from(pe).unwrap();
/// ```
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Geometry<T: CoordNum = f64, Z: ZCoord = NoValue, M: Measure = NoValue> {
    Point(Point<T, Z, M>),
    Line(Line<T, Z, M>),
    LineString(LineString<T, Z, M>),
    Polygon(Polygon<T, Z, M>),
    MultiPoint(MultiPoint<T, Z, M>),
    MultiLineString(MultiLineString<T, Z, M>),
    MultiPolygon(MultiPolygon<T, Z, M>),
    GeometryCollection(GeometryCollection<T, Z, M>),
    Rect(Rect<T, Z, M>),
    Triangle(Triangle<T, Z, M>),
}

pub type GeometryM<T> = Geometry<T, NoValue, T>;
pub type Geometry3D<T> = Geometry<T, T, NoValue>;
pub type Geometry3DM<T> = Geometry<T, T, T>;

impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Point<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: Point<T, Z, M>) -> Self {
        Self::Point(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Line<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: Line<T, Z, M>) -> Self {
        Self::Line(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<LineString<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: LineString<T, Z, M>) -> Self {
        Self::LineString(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Polygon<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: Polygon<T, Z, M>) -> Self {
        Self::Polygon(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<MultiPoint<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: MultiPoint<T, Z, M>) -> Self {
        Self::MultiPoint(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<MultiLineString<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: MultiLineString<T, Z, M>) -> Self {
        Self::MultiLineString(x)
    }
}
impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<MultiPolygon<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: MultiPolygon<T, Z, M>) -> Self {
        Self::MultiPolygon(x)
    }
}

// Disabled until we remove the deprecated GeometryCollection::from(single_geom) impl.
// impl<T: CoordNum, Z: ZCoord, M: Measure> From<GeometryCollection<T, Z, M>> for Geometry<T, Z, M> {
//     fn from(x: GeometryCollection<T, Z, M>) -> Self {
//         Self::GeometryCollection(x)
//     }
// }

impl<T: CoordNum, Z: ZCoord, M: Measure> From<Rect<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: Rect<T, Z, M>) -> Self {
        Self::Rect(x)
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> From<Triangle<T, Z, M>> for Geometry<T, Z, M> {
    fn from(x: Triangle<T, Z, M>) -> Self {
        Self::Triangle(x)
    }
}

macro_rules! try_from_geometry_impl {
    ($($type: ident),+ $(,)? ) => {
        $(
        /// Convert a Geometry enum into its inner type.
        ///
        /// Fails if the enum case does not match the type you are trying to convert it to.
        impl<T: CoordNum, Z: CoordNum, M: CoordNum> TryFrom<Geometry<T, Z, M>> for $type<T, Z, M> {
            type Error = Error;

            fn try_from(geom: Geometry<T, Z, M>) -> Result<Self, Self::Error> {
                match geom {
                    Geometry::$type(g) => Ok(g),
                    other => Err(Error::MismatchedGeometry {
                        expected: type_name::<$type<T, Z, M>>(),
                        found: inner_type_name(other)
                    })
                }
            }
        }
        )+
    }
}

// `concat_idents` is not available, so hacking around it
try_from_geometry_impl!(
    Point,
    Line,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    // Disabled until we remove the deprecated GeometryCollection::from(single_geom) impl.
    // GeometryCollection,
    Rect,
    Triangle,
);

fn inner_type_name<T: CoordNum, Z: CoordNum, M: CoordNum>(
    geometry: Geometry<T, Z, M>,
) -> &'static str {
    match geometry {
        Geometry::Point(_) => type_name::<Point<T, Z, M>>(),
        Geometry::Line(_) => type_name::<Line<T, Z, M>>(),
        Geometry::LineString(_) => type_name::<LineString<T, Z, M>>(),
        Geometry::Polygon(_) => type_name::<Polygon<T, Z, M>>(),
        Geometry::MultiPoint(_) => type_name::<MultiPoint<T, Z, M>>(),
        Geometry::MultiLineString(_) => type_name::<MultiLineString<T, Z, M>>(),
        Geometry::MultiPolygon(_) => type_name::<MultiPolygon<T, Z, M>>(),
        Geometry::GeometryCollection(_) => type_name::<GeometryCollection<T, Z, M>>(),
        Geometry::Rect(_) => type_name::<Rect<T, Z, M>>(),
        Geometry::Triangle(_) => type_name::<Triangle<T, Z, M>>(),
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Geometry<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// Equality assertion within a relative limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Geometry, polygon};
    ///
    /// let a: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)].into();
    /// let b: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)].into();
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// approx::assert_relative_ne!(a, b, max_relative=0.001);
    /// ```
    ///
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::GeometryCollection(g1), Geometry::GeometryCollection(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Rect(g1), Geometry::Rect(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (_, _) => false,
        }
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Geometry<T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// Equality assertion with an absolute limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Geometry, polygon};
    ///
    /// let a: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)].into();
    /// let b: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)].into();
    ///
    /// approx::assert_abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::assert_abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.abs_diff_eq(g2, epsilon)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::GeometryCollection(g1), Geometry::GeometryCollection(g2)) => {
                g1.abs_diff_eq(g2, epsilon)
            }
            (Geometry::Rect(g1), Geometry::Rect(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => g1.abs_diff_eq(g2, epsilon),
            (_, _) => false,
        }
    }
}
