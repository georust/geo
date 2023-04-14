pub(crate) mod coord;
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
#[allow(deprecated)]
pub use coord::{Coord, Coordinate};
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
use core::convert::TryFrom;

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
/// let p = point!(x: 1.0, y: 1.0);
/// let pe: Geometry = p.into();
/// let pn = Point::try_from(pe).unwrap();
/// ```
///
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Geometry<T: CoordNum = f64> {
    Point(Point<T>),
    Line(Line<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
    Rect(Rect<T>),
    Triangle(Triangle<T>),
}

impl<T: CoordNum> From<Point<T>> for Geometry<T> {
    fn from(x: Point<T>) -> Self {
        Self::Point(x)
    }
}
impl<T: CoordNum> From<Line<T>> for Geometry<T> {
    fn from(x: Line<T>) -> Self {
        Self::Line(x)
    }
}
impl<T: CoordNum> From<LineString<T>> for Geometry<T> {
    fn from(x: LineString<T>) -> Self {
        Self::LineString(x)
    }
}
impl<T: CoordNum> From<Polygon<T>> for Geometry<T> {
    fn from(x: Polygon<T>) -> Self {
        Self::Polygon(x)
    }
}
impl<T: CoordNum> From<MultiPoint<T>> for Geometry<T> {
    fn from(x: MultiPoint<T>) -> Self {
        Self::MultiPoint(x)
    }
}
impl<T: CoordNum> From<MultiLineString<T>> for Geometry<T> {
    fn from(x: MultiLineString<T>) -> Self {
        Self::MultiLineString(x)
    }
}
impl<T: CoordNum> From<MultiPolygon<T>> for Geometry<T> {
    fn from(x: MultiPolygon<T>) -> Self {
        Self::MultiPolygon(x)
    }
}

// Disabled until we remove the deprecated GeometryCollection::from(single_geom) impl.
// impl<T: CoordNum> From<GeometryCollection<T>> for Geometry<T> {
//     fn from(x: GeometryCollection<T>) -> Self {
//         Self::GeometryCollection(x)
//     }
// }

impl<T: CoordNum> From<Rect<T>> for Geometry<T> {
    fn from(x: Rect<T>) -> Self {
        Self::Rect(x)
    }
}

impl<T: CoordNum> From<Triangle<T>> for Geometry<T> {
    fn from(x: Triangle<T>) -> Self {
        Self::Triangle(x)
    }
}

impl<T: CoordNum> Geometry<T> {
    /// If this Geometry is a Point, then return that, else None.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::*;
    /// use std::convert::TryInto;
    ///
    /// let g = Geometry::Point(Point::new(0., 0.));
    /// let p2: Point<f32> = g.try_into().unwrap();
    /// assert_eq!(p2, Point::new(0., 0.,));
    /// ```
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Point>"
    )]
    pub fn into_point(self) -> Option<Point<T>> {
        if let Geometry::Point(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a LineString, then return that LineString, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<LineString>"
    )]
    pub fn into_line_string(self) -> Option<LineString<T>> {
        if let Geometry::LineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Line, then return that Line, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Line>"
    )]
    pub fn into_line(self) -> Option<Line<T>> {
        if let Geometry::Line(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Polygon, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Polygon>"
    )]
    pub fn into_polygon(self) -> Option<Polygon<T>> {
        if let Geometry::Polygon(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPoint, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiPoint>"
    )]
    pub fn into_multi_point(self) -> Option<MultiPoint<T>> {
        if let Geometry::MultiPoint(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiLineString, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiLineString>"
    )]
    pub fn into_multi_line_string(self) -> Option<MultiLineString<T>> {
        if let Geometry::MultiLineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPolygon, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiPolygon>"
    )]
    pub fn into_multi_polygon(self) -> Option<MultiPolygon<T>> {
        if let Geometry::MultiPolygon(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

macro_rules! try_from_geometry_impl {
    ($($type: ident),+) => {
        $(
        /// Convert a Geometry enum into its inner type.
        ///
        /// Fails if the enum case does not match the type you are trying to convert it to.
        impl <T: CoordNum> TryFrom<Geometry<T>> for $type<T> {
            type Error = Error;

            fn try_from(geom: Geometry<T>) -> Result<Self, Self::Error> {
                match geom {
                    Geometry::$type(g) => Ok(g),
                    other => Err(Error::MismatchedGeometry {
                        expected: type_name::<$type<T>>(),
                        found: inner_type_name(other)
                    })
                }
            }
        }
        )+
    }
}

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
    Triangle
);

fn inner_type_name<T>(geometry: Geometry<T>) -> &'static str
where
    T: CoordNum,
{
    match geometry {
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

#[cfg(any(feature = "rstar_0_8", feature = "rstar_0_9", feature = "rstar_0_10"))]
macro_rules! impl_rstar_geometry {
    ($rstar:ident) => {
        impl<T> $rstar::RTreeObject for Geometry<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                match self {
                    Self::Point(x) => x.envelope(),
                    Self::Line(x) => x.envelope(),
                    Self::LineString(x) => x.envelope(),
                    Self::Polygon(x) => x.envelope(),
                    Self::Rect(x) => x.envelope(),
                    Self::Triangle(x) => x.envelope(),
                    Self::MultiPoint(x) => x.envelope(),
                    Self::MultiLineString(x) => x.envelope(),
                    Self::MultiPolygon(x) => x.envelope(),
                    Self::GeometryCollection(x) => x.envelope(),
                }
            }
        }
    };
}

#[cfg(feature = "rstar_0_8")]
impl_rstar_geometry!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_geometry!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_geometry!(rstar_0_10);

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

/// Implements the common pattern where a Geometry enum simply delegates its trait impl to it's inner type.
///
/// ```
/// # use geo_types::{CoordNum, Coord, Point, Line, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, GeometryCollection, Rect, Triangle, Geometry};
///
/// trait Foo<T: CoordNum> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool;
///     fn foo_2(&self) -> i32;
/// }
///
/// // Assuming we have an impl for all the inner types like this:
/// impl<T: CoordNum> Foo<T> for Point<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 1 }
/// }
/// impl<T: CoordNum> Foo<T> for Line<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 2 }
/// }
/// impl<T: CoordNum> Foo<T> for LineString<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 3 }
/// }
/// impl<T: CoordNum> Foo<T> for Polygon<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 4 }
/// }
/// impl<T: CoordNum> Foo<T> for MultiPoint<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 5 }
/// }
/// impl<T: CoordNum> Foo<T> for MultiLineString<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 6 }
/// }
/// impl<T: CoordNum> Foo<T> for MultiPolygon<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 7 }
/// }
/// impl<T: CoordNum> Foo<T> for GeometryCollection<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 8 }
/// }
/// impl<T: CoordNum> Foo<T> for Rect<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 9 }
/// }
/// impl<T: CoordNum> Foo<T> for Triangle<T> {
///     fn foo_1(&self, coord: Coord<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 10 }
/// }
///
/// // If we want the impl for Geometry to simply delegate to it's
/// // inner case...
/// impl<T: CoordNum> Foo<T> for Geometry<T> {
///     // Instead of writing out this trivial enum delegation...
///     // fn foo_1(&self, coord: Coord<T>)  -> bool {
///     //     match self {
///     //        Geometry::Point(g) => g.foo_1(coord),
///     //        Geometry::LineString(g) => g.foo_1(coord),
///     //        _ => unimplemented!("...etc for other cases")
///     //     }
///     // }
///     //
///     // fn foo_2(&self)  -> i32 {
///     //     match self {
///     //        Geometry::Point(g) => g.foo_2(),
///     //        Geometry::LineString(g) => g.foo_2(),
///     //        _ => unimplemented!("...etc for other cases")
///     //     }
///     // }
///
///     // we can equivalently write:
///     geo_types::geometry_delegate_impl! {
///         fn foo_1(&self, coord: Coord<T>) -> bool;
///         fn foo_2(&self) -> i32;
///     }
/// }
/// ```
#[macro_export]
macro_rules! geometry_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ Geometry, $($a)* } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! geometry_cow_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ GeometryCow, $($a)* } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __geometry_delegate_impl_helper {
    (
        $enum:ident,
        $(
            $(#[$outer:meta])*
            fn $func_name: ident(&$($self_life:lifetime)?self $(, $arg_name: ident: $arg_type: ty)*) -> $return: ty;
         )+
    ) => {
            $(
                $(#[$outer])*
                fn $func_name(&$($self_life)? self, $($arg_name: $arg_type),*) -> $return {
                    match self {
                        $enum::Point(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Line(g) =>  g.$func_name($($arg_name),*).into(),
                        $enum::LineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Polygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPoint(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiLineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPolygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::GeometryCollection(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Rect(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Triangle(g) => g.$func_name($($arg_name),*).into(),
                    }
                }
            )+
        };
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "rstar_0_9")]
    use super::*;
    #[cfg(feature = "rstar_0_9")]
    use crate::point;
    #[cfg(feature = "rstar_0_9")]
    use rstar_0_9::{RTree, AABB};

    #[test]
    #[cfg(feature = "rstar_0_9")]
    fn test_geometry_enum_rstar() {
        let mut tree = RTree::new();
        let p1 = point!(x: 1.0, y: 1.0);
        let p2 = point!(x: 2.0, y: 2.0);
        let p3 = point!(x: 3.0, y: 3.0);
        let pg1: Geometry = p1.into();
        let pg2: Geometry = p2.into();
        let pg3: Geometry = p3.into();
        tree.insert(pg1);
        tree.insert(pg2);
        tree.insert(pg3);
        let qp = point!(x: 2.0, y: 2.0);
        let found = tree.locate_in_envelope(&AABB::from_point(qp)).next();
        assert_eq!(found.unwrap(), &point!(x: 2.0, y: 2.0).into());
    }
}
