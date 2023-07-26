use crate::GeoFloat;
use crate::CoordsIter;
use crate::algorithm::EuclideanDistance;
use num_traits::Bounded; // used to have T as generic type in folding
use geo_types::*;

/// Determine the distance between two geometries using the [Hausdorff distance formula].
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Hausdorff_distance
pub trait HausdorffDistance<T, Rhs = Self> {
    fn hausdorff_distance(&self, rhs: &Rhs) -> T;
}

// We can take advantage of the coords_iter() method for all geometries
// to iterate across all combos of coords (infinum). Simplifies the 
// implementations for all geometries.
macro_rules! impl_hausdorff_distance_coord_iter {
    ($from:ident, [$($to:ident),*]) => {
        $(
            impl<T> HausdorffDistance<T, $to<T>> for $from<T>
            where
                T: GeoFloat 
            {
                fn hausdorff_distance(&self, geom: &$to<T>) -> T {
                    // calculate from A -> B 
                    let hd1 = self
                        .coords_iter()
                        .map(|c| {
                            geom
                            .coords_iter()
                            .map(|c2| {
                                c.euclidean_distance(&c2)
                            })
                            .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val))
                        })
                        .fold(<T as Bounded>::min_value(), |accum, val| accum.max(val));
            
                    // Calculate from B -> A
                    let hd2 = geom
                        .coords_iter()
                        .map(|c| {
                            self
                            .coords_iter()
                            .map(|c2| {
                                c.euclidean_distance(&c2)
                            })
                            .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val))
                        })
                        .fold(<T as Bounded>::min_value(), |accum, val| accum.max(val));
            
                    // The max of the two
                    hd1.max(hd2)
                }
            }
        )*
    };
}


impl_hausdorff_distance_coord_iter!{
    Line, 
    [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}
    

impl_hausdorff_distance_coord_iter!{
    Triangle, 
    [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    Rect, 
    [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    Point, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    LineString, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    Polygon, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}


impl_hausdorff_distance_coord_iter!{
    MultiPoint, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    MultiLineString, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}
    
impl_hausdorff_distance_coord_iter!{
    MultiPolygon, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}
    
impl_hausdorff_distance_coord_iter!{
    GeometryCollection, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter!{
    Geometry, [
        Line, Rect, Triangle, Point, MultiPoint, 
        LineString, MultiLineString, 
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}


// ┌───────────────────────────┐
// │ Implementations for Coord │
// └───────────────────────────┘

// since Coord does not have coords_iter() method we have 
// to make a macro for it specifically
macro_rules! impl_haussdorf_distance_coord {
    ($($for:ident),*) => {
        $(
            impl<T> HausdorffDistance<T, $for<T>> for Coord<T>
            where
                T: GeoFloat 
            {
                fn hausdorff_distance(&self, geom: &$for<T>) -> T {
                    let p = Point::from(*self);
                    p.hausdorff_distance(geom)
                }
            }
        )*
    };
}

// Implement methods for all other geometries
impl_haussdorf_distance_coord!(
    Line, Rect, Triangle, 
    Point, MultiPoint, 
    LineString, MultiLineString, 
    Polygon, MultiPolygon,
    Geometry, GeometryCollection
);
