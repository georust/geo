use crate::{
    CoordNum, Error, GeometryCollectionTZM, LineStringTZM, LineTZM, Measure, MultiLineStringTZM,
    MultiPointTZM, MultiPolygonTZM, NoValue, PointTZM, PolygonTZM, RectTZM, TriangleTZM, ZCoord,
};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use core::any::type_name;
use std::convert::TryFrom;

/// An enum representing any possible geometry type.
///
/// All `Geo` types can be converted to a `Geometry` member using `.into()` (as part of the
/// `std::convert::Into` pattern), and `Geo` types implement the `TryFrom` trait in order to
/// convert _back_ from enum members.
///
/// # Example
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe: Geometry<f64> = p.into();
/// let pn = Point::try_from(pe).unwrap();
/// ```
///
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GeometryTZM<T: CoordNum, Z: ZCoord, M: Measure> {
    Point(PointTZM<T, Z, M>),
    Line(LineTZM<T, Z, M>),
    LineString(LineStringTZM<T, Z, M>),
    Polygon(PolygonTZM<T, Z, M>),
    MultiPoint(MultiPointTZM<T, Z, M>),
    MultiLineString(MultiLineStringTZM<T, Z, M>),
    MultiPolygon(MultiPolygonTZM<T, Z, M>),
    GeometryCollection(GeometryCollectionTZM<T, Z, M>),
    Rect(RectTZM<T, Z, M>),
    Triangle(TriangleTZM<T, Z, M>),
}

pub type Geometry<T> = GeometryTZM<T, NoValue, NoValue>;
pub type GeometryM<T, M> = GeometryTZM<T, NoValue, M>;
pub type GeometryZ<T> = GeometryTZM<T, T, NoValue>;
pub type GeometryZM<T, M> = GeometryTZM<T, T, M>;

impl<T: CoordNum, Z: ZCoord, M: Measure> From<PointTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: PointTZM<T, Z, M>) -> Self {
        Self::Point(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<LineTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: LineTZM<T, Z, M>) -> Self {
        Self::Line(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<LineStringTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: LineStringTZM<T, Z, M>) -> Self {
        Self::LineString(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<PolygonTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: PolygonTZM<T, Z, M>) -> Self {
        Self::Polygon(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<MultiPointTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: MultiPointTZM<T, Z, M>) -> Self {
        Self::MultiPoint(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<MultiLineStringTZM<T, Z, M>>
    for GeometryTZM<T, Z, M>
{
    fn from(x: MultiLineStringTZM<T, Z, M>) -> Self {
        Self::MultiLineString(x)
    }
}
impl<T: CoordNum, Z: ZCoord, M: Measure> From<MultiPolygonTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: MultiPolygonTZM<T, Z, M>) -> Self {
        Self::MultiPolygon(x)
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> From<RectTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: RectTZM<T, Z, M>) -> Self {
        Self::Rect(x)
    }
}

impl<T: CoordNum, Z: ZCoord, M: Measure> From<TriangleTZM<T, Z, M>> for GeometryTZM<T, Z, M> {
    fn from(x: TriangleTZM<T, Z, M>) -> Self {
        Self::Triangle(x)
    }
}

macro_rules! try_from_geometry_impl {
    ($(($type: ident, $typeTZM: ident)),+ $(,)? ) => {
        $(
        /// Convert a Geometry enum into its inner type.
        ///
        /// Fails if the enum case does not match the type you are trying to convert it to.
        impl <T: CoordNum, Z: ZCoord, M: Measure> TryFrom<GeometryTZM<T, Z, M>> for $typeTZM<T, Z, M> {
            type Error = Error;

            fn try_from(geom: GeometryTZM<T, Z, M>) -> Result<Self, Self::Error> {
                match geom {
                    GeometryTZM::$type(g) => Ok(g),
                    other => Err(Error::MismatchedGeometry {
                        expected: type_name::<$typeTZM<T, Z, M>>(),
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
    (Point, PointTZM),
    (Line, LineTZM),
    (LineString, LineStringTZM),
    (Polygon, PolygonTZM),
    (MultiPoint, MultiPointTZM),
    (MultiLineString, MultiLineStringTZM),
    (MultiPolygon, MultiPolygonTZM),
    (Rect, RectTZM),
    (Triangle, TriangleTZM),
);

fn inner_type_name<T: CoordNum, Z: ZCoord, M: Measure>(
    geometry: GeometryTZM<T, Z, M>,
) -> &'static str {
    match geometry {
        GeometryTZM::Point(_) => type_name::<PointTZM<T, Z, M>>(),
        GeometryTZM::Line(_) => type_name::<LineTZM<T, Z, M>>(),
        GeometryTZM::LineString(_) => type_name::<LineStringTZM<T, Z, M>>(),
        GeometryTZM::Polygon(_) => type_name::<PolygonTZM<T, Z, M>>(),
        GeometryTZM::MultiPoint(_) => type_name::<MultiPointTZM<T, Z, M>>(),
        GeometryTZM::MultiLineString(_) => type_name::<MultiLineStringTZM<T, Z, M>>(),
        GeometryTZM::MultiPolygon(_) => type_name::<MultiPolygonTZM<T, Z, M>>(),
        GeometryTZM::GeometryCollection(_) => type_name::<GeometryCollectionTZM<T, Z, M>>(),
        GeometryTZM::Rect(_) => type_name::<RectTZM<T, Z, M>>(),
        GeometryTZM::Triangle(_) => type_name::<TriangleTZM<T, Z, M>>(),
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
