use crate::{GeoFloat, Point};

/// The result of trying to find the closest spot on an object to a point.
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Closest<F: GeoFloat> {
    /// The point actually intersects with the object.
    Intersection(Point<F>),
    /// There is exactly one place on this object which is closest to the point.
    SinglePoint(Point<F>),
    /// There are two or more (possibly infinite or undefined) possible points.
    Indeterminate,
}

impl<F: GeoFloat> Closest<F> {
    /// Compare two `Closest`s relative to `p` and return a copy of the best
    /// one.
    pub fn best_of_two(&self, other: &Self, p: Point<F>) -> Self {
        use crate::algorithm::euclidean_distance::EuclideanDistance;

        let left = match *self {
            Closest::Indeterminate => return *other,
            Closest::Intersection(_) => return *self,
            Closest::SinglePoint(l) => l,
        };
        let right = match *other {
            Closest::Indeterminate => return *self,
            Closest::Intersection(_) => return *other,
            Closest::SinglePoint(r) => r,
        };

        if left.euclidean_distance(&p) <= right.euclidean_distance(&p) {
            *self
        } else {
            *other
        }
    }
}

/// Implements the common pattern where a Geometry enum simply delegates its trait impl to it's inner type.
///
/// ```
/// # use geo::{GeoNum, Coordinate, Point, Line, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, GeometryCollection, Rect, Triangle, Geometry};
///
/// trait Foo<T: GeoNum> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool;
///     fn foo_2(&self) -> i32;
/// }
///
/// // Assuming we have an impl for all the inner types like this:
/// impl<T: GeoNum> Foo<T> for Point<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 1 }
/// }
/// impl<T: GeoNum> Foo<T> for Line<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 2 }
/// }
/// impl<T: GeoNum> Foo<T> for LineString<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 3 }
/// }
/// impl<T: GeoNum> Foo<T> for Polygon<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 4 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiPoint<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 5 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiLineString<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 6 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiPolygon<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 7 }
/// }
/// impl<T: GeoNum> Foo<T> for GeometryCollection<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { false }
///     fn foo_2(&self)  -> i32 { 8 }
/// }
/// impl<T: GeoNum> Foo<T> for Rect<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 9 }
/// }
/// impl<T: GeoNum> Foo<T> for Triangle<T> {
///     fn foo_1(&self, coord: Coordinate<T>)  -> bool { true }
///     fn foo_2(&self)  -> i32 { 10 }
/// }
///
/// // If we want the impl for Geometry to simply delegate to it's
/// // inner case...
/// impl<T: GeoNum> Foo<T> for Geometry<T> {
///     // Instead of writing out this trivial enum delegation...
///     // fn foo_1(&self, coord: Coordinate<T>)  -> bool {
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
///     geo::geometry_delegate_impl! {
///         fn foo_1(&self, coord: Coordinate<T>) -> bool;
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
