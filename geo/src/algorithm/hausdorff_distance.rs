use crate::algorithm::EuclideanDistance;
use crate::CoordsIter;
use crate::GeoFloat;
use geo_types::*;
use num_traits::Bounded; // used to have T as generic type in folding

/// Determine the distance between two geometries using the [Hausdorff distance formula].
///
/// Hausdorff distance is used to compare two point sets. It measures the maximum euclidean
/// distance of a point in one set to the nearest point in another set. Hausdorff distance
/// is often used to measure the amount of mismatch between two sets.
///
/// [Hausdorff distance formula]: https://en.wikipedia.org/wiki/Hausdorff_distance
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

impl_hausdorff_distance_coord_iter! {
    Line, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    Triangle, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    Rect, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    Point, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    LineString, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    Polygon, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    MultiPoint, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    MultiLineString, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    MultiPolygon, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
    GeometryCollection, [
        Line, Rect, Triangle, Point, MultiPoint,
        LineString, MultiLineString,
        Polygon, MultiPolygon,
        Geometry, GeometryCollection
    ]
}

impl_hausdorff_distance_coord_iter! {
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
    Line,
    Rect,
    Triangle,
    Point,
    MultiPoint,
    LineString,
    MultiLineString,
    Polygon,
    MultiPolygon,
    Geometry,
    GeometryCollection
);

#[cfg(test)]
mod test {
    use crate::HausdorffDistance;
    use crate::{line_string, polygon, MultiPoint, MultiPolygon};

    #[test]
    fn hd_mpnt_mpnt() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let p2: MultiPoint<_> = vec![(2., 3.), (1., 2.)].into();
        assert_relative_eq!(p1.hausdorff_distance(&p2), 2.236068, epsilon = 1.0e-6);
    }

    #[test]
    fn hd_mpnt_poly() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let poly = polygon![
        (x: 1., y: -3.1), (x: 3.7, y: 2.7),
        (x: 0.9, y: 7.6), (x: -4.8, y: 6.7),
        (x: -7.5, y: 0.9), (x: -4.7, y: -4.),
        (x: 1., y: -3.1)
        ];

        assert_relative_eq!(p1.hausdorff_distance(&poly), 7.553807, epsilon = 1.0e-6)
    }

    #[test]
    fn hd_mpnt_lns() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let lns = line_string![
        (x: 1., y: -3.1), (x: 3.7, y: 2.7),
        (x: 0.9, y: 7.6), (x: -4.8, y: 6.7),
        (x: -7.5, y: 0.9), (x: -4.7, y: -4.),
        (x: 1., y: -3.1)
        ];

        assert_relative_eq!(p1.hausdorff_distance(&lns), 7.553807, epsilon = 1.0e-6)
    }

    #[test]
    fn hd_mpnt_mply() {
        let p1: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let multi_polygon = MultiPolygon::new(vec![
            polygon![
              (x: 0.0f32, y: 0.0),
              (x: 2.0, y: 0.0),
              (x: 2.0, y: 1.0),
              (x: 0.0, y: 1.0),
            ],
            polygon![
              (x: 1.0, y: 1.0),
              (x: -2.0, y: 1.0),
              (x: -2.0, y: -1.0),
              (x: 1.0, y: -1.0),
            ],
        ]);

        assert_relative_eq!(
            p1.hausdorff_distance(&multi_polygon),
            2.236068,
            epsilon = 1.0e-6
        )
    }
}
