use super::{Distance, Euclidean};
use crate::algorithm::Intersects;
use crate::coordinate_position::{coord_pos_relative_to_ring, CoordPos};
use crate::geometry::*;
use crate::{CoordFloat, GeoFloat, GeoNum};
use num_traits::{Bounded, Float};
use rstar::primitives::CachedEnvelope;
use rstar::RTree;

// Distance is a symmetric operation, so we can implement it once for both
macro_rules! symmetric_distance_impl {
    ($t:ident, $a:ty, $b:ty) => {
        impl<F> $crate::Distance<F, $a, $b> for Euclidean
        where
            F: $t,
        {
            fn distance(&self, a: $a, b: $b) -> F {
                self.distance(b, a)
            }
        }
    };
}

// ┌───────────────────────────┐
// │ Implementations for Coord │
// └───────────────────────────┘

impl<F: CoordFloat> Distance<F, Coord<F>, Coord<F>> for Euclidean {
    fn distance(&self, origin: Coord<F>, destination: Coord<F>) -> F {
        let delta = origin - destination;
        delta.x.hypot(delta.y)
    }
}
impl<F: CoordFloat> Distance<F, Coord<F>, &Line<F>> for Euclidean {
    fn distance(&self, coord: Coord<F>, line: &Line<F>) -> F {
        ::geo_types::private_utils::point_line_euclidean_distance(Point(coord), *line)
    }
}

// ┌───────────────────────────┐
// │ Implementations for Point │
// └───────────────────────────┘

/// Calculate the Euclidean distance (a.k.a. pythagorean distance) between two Points
impl<F: CoordFloat> Distance<F, Point<F>, Point<F>> for Euclidean {
    /// Calculate the Euclidean distance (a.k.a. pythagorean distance) between two Points
    ///
    /// # Units
    /// - `origin`, `destination`: Point where the units of x/y represent non-angular units
    ///   — e.g. meters or miles, not lon/lat. For lon/lat points, use the
    ///   [`Haversine`] or [`Geodesic`] [metric spaces].
    /// - returns: distance in the same units as the `origin` and `destination` points
    ///
    /// # Example
    /// ```
    /// use geo::{Euclidean, Distance};
    /// use geo::Point;
    /// // web mercator
    /// let new_york_city = Point::new(-8238310.24, 4942194.78);
    /// // web mercator
    /// let london = Point::new(-14226.63, 6678077.70);
    /// let distance: f64 = Euclidean.distance(new_york_city, london);
    ///
    /// assert_eq!(
    ///     8_405_286., // meters in web mercator
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [`Haversine`]: crate::line_measures::metric_spaces::Haversine
    /// [`Geodesic`]: crate::line_measures::metric_spaces::Geodesic
    /// [metric spaces]: crate::line_measures::metric_spaces
    fn distance(&self, origin: Point<F>, destination: Point<F>) -> F {
        self.distance(origin.0, destination.0)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &Point<F>> for Euclidean {
    fn distance(&self, origin: &Point<F>, destination: &Point<F>) -> F {
        self.distance(*origin, *destination)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &Line<F>> for Euclidean {
    fn distance(&self, origin: &Point<F>, destination: &Line<F>) -> F {
        geo_types::private_utils::point_line_euclidean_distance(*origin, *destination)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &LineString<F>> for Euclidean {
    fn distance(&self, origin: &Point<F>, destination: &LineString<F>) -> F {
        geo_types::private_utils::point_line_string_euclidean_distance(*origin, destination)
    }
}

impl<F: GeoFloat> Distance<F, &Point<F>, &Polygon<F>> for Euclidean {
    fn distance(&self, point: &Point<F>, polygon: &Polygon<F>) -> F {
        // No need to continue if the polygon intersects the point, or is zero-length
        if polygon.exterior().0.is_empty() || polygon.intersects(point) {
            return F::zero();
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors()
            .iter()
            .map(|ring| self.distance(point, ring))
            .fold(Bounded::max_value(), |accum: F, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| {
                        ::geo_types::private_utils::line_segment_distance(
                            point.0, line.start, line.end,
                        )
                    })
                    .fold(Bounded::max_value(), |accum, val| accum.min(val)),
            )
    }
}

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

symmetric_distance_impl!(CoordFloat, &Line<F>, Coord<F>);
symmetric_distance_impl!(CoordFloat, &Line<F>, &Point<F>);

impl<F: GeoFloat> Distance<F, &Line<F>, &Line<F>> for Euclidean {
    fn distance(&self, line_a: &Line<F>, line_b: &Line<F>) -> F {
        if line_a.intersects(line_b) {
            return F::zero();
        }
        // minimum of all Point-Line distances
        self.distance(&line_a.start_point(), line_b)
            .min(self.distance(&line_a.end_point(), line_b))
            .min(self.distance(&line_b.start_point(), line_a))
            .min(self.distance(&line_b.end_point(), line_a))
    }
}

impl<F: GeoFloat> Distance<F, &Line<F>, &LineString<F>> for Euclidean {
    fn distance(&self, line: &Line<F>, line_string: &LineString<F>) -> F {
        line_string
            .lines()
            .fold(Bounded::max_value(), |acc, segment| {
                acc.min(self.distance(line, &segment))
            })
    }
}

impl<F: GeoFloat> Distance<F, &Line<F>, &Polygon<F>> for Euclidean {
    fn distance(&self, line: &Line<F>, polygon: &Polygon<F>) -> F {
        if line.intersects(polygon) {
            return F::zero();
        }

        std::iter::once(polygon.exterior())
            .chain(polygon.interiors().iter())
            .fold(Bounded::max_value(), |acc, line_string| {
                acc.min(self.distance(line, line_string))
            })
    }
}

// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

symmetric_distance_impl!(CoordFloat, &LineString<F>, &Point<F>);
symmetric_distance_impl!(GeoFloat, &LineString<F>, &Line<F>);

impl<F: GeoFloat> Distance<F, &LineString<F>, &LineString<F>> for Euclidean {
    fn distance(&self, line_string_a: &LineString<F>, line_string_b: &LineString<F>) -> F {
        if line_string_a.intersects(line_string_b) {
            F::zero()
        } else {
            nearest_neighbour_distance(line_string_a, line_string_b)
        }
    }
}

impl<F: GeoFloat> Distance<F, &LineString<F>, &Polygon<F>> for Euclidean {
    fn distance(&self, line_string: &LineString<F>, polygon: &Polygon<F>) -> F {
        if line_string.intersects(polygon) {
            F::zero()
        } else if !polygon.interiors().is_empty()
            // FIXME: Explodes on empty line_string
            && ring_contains_coord(polygon.exterior(), line_string.0[0])
        {
            // check each ring distance, returning the minimum
            let mut mindist: F = Float::max_value();
            for ring in polygon.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(line_string, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance(line_string, polygon.exterior())
        }
    }
}

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘

symmetric_distance_impl!(GeoFloat, &Polygon<F>, &Point<F>);
symmetric_distance_impl!(GeoFloat, &Polygon<F>, &Line<F>);
symmetric_distance_impl!(GeoFloat, &Polygon<F>, &LineString<F>);
impl<F: GeoFloat> Distance<F, &Polygon<F>, &Polygon<F>> for Euclidean {
    fn distance(&self, polygon_a: &Polygon<F>, polygon_b: &Polygon<F>) -> F {
        if polygon_a.intersects(polygon_b) {
            return F::zero();
        }
        // FIXME: explodes when polygon_b.exterior() is empty
        // Containment check
        if !polygon_a.interiors().is_empty()
            && ring_contains_coord(polygon_a.exterior(), polygon_b.exterior().0[0])
        {
            // check each ring distance, returning the minimum
            let mut mindist: F = Float::max_value();
            for ring in polygon_a.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(polygon_b.exterior(), ring))
            }
            return mindist;
        } else if !polygon_b.interiors().is_empty()
            // FIXME: explodes when polygon_a.exterior() is empty
            && ring_contains_coord(polygon_b.exterior(), polygon_a.exterior().0[0])
        {
            let mut mindist: F = Float::max_value();
            for ring in polygon_b.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(polygon_a.exterior(), ring))
            }
            return mindist;
        }
        nearest_neighbour_distance(polygon_a.exterior(), polygon_b.exterior())
    }
}

// ┌────────────────────────────────────────┐
// │ Implementations for Rect and Triangle  │
// └────────────────────────────────────────┘

/// Implements Euclidean distance for Triangles and Rects by converting them to polygons.
macro_rules! impl_euclidean_distance_for_polygonlike_geometry {
  ($polygonlike:ty,  [$($geometry_b:ty),*]) => {
      impl<F: GeoFloat> Distance<F, $polygonlike, $polygonlike> for Euclidean
      {
          fn distance(&self, origin: $polygonlike, destination: $polygonlike) -> F {
              self.distance(&origin.to_polygon(), destination)
          }
      }
      $(
          impl<F: GeoFloat> Distance<F, $polygonlike, $geometry_b> for Euclidean
          {
              fn distance(&self, polygonlike: $polygonlike, geometry_b: $geometry_b) -> F {
                    self.distance(&polygonlike.to_polygon(), geometry_b)
              }
          }
          symmetric_distance_impl!(GeoFloat, $geometry_b, $polygonlike);
      )*
  };
}

impl_euclidean_distance_for_polygonlike_geometry!(&Triangle<F>,  [&Point<F>, &Line<F>, &LineString<F>, &Polygon<F>, &Rect<F>]);
impl_euclidean_distance_for_polygonlike_geometry!(&Rect<F>,      [&Point<F>, &Line<F>, &LineString<F>, &Polygon<F>]);

// ┌───────────────────────────────────────────┐
// │ Implementations for multi geometry types  │
// └───────────────────────────────────────────┘

/// Euclidean distance implementation for multi geometry types.
macro_rules! impl_euclidean_distance_for_iter_geometry {
    ($iter_geometry:ty,  [$($to_geometry:ty),*]) => {
        impl<F: GeoFloat> Distance<F, $iter_geometry, $iter_geometry> for Euclidean {
            fn distance(&self, origin: $iter_geometry, destination: $iter_geometry) -> F {
                origin
                    .iter()
                    .fold(Bounded::max_value(), |accum: F, member| {
                        accum.min(self.distance(member, destination))
                    })
             }
        }
        $(
            impl<F: GeoFloat> Distance<F, $iter_geometry, $to_geometry> for Euclidean {
                fn distance(&self, iter_geometry: $iter_geometry, to_geometry: $to_geometry) -> F {
                    iter_geometry
                        .iter()
                        .fold(Bounded::max_value(), |accum: F, member| {
                            accum.min(self.distance(member, to_geometry))
                        })
                }
            }
            symmetric_distance_impl!(GeoFloat, $to_geometry, $iter_geometry);
        )*
  };
}

impl_euclidean_distance_for_iter_geometry!(&MultiPoint<F>,         [&Point<F>, &Line<F>, &LineString<F>, &MultiLineString<F>, &Polygon<F>, &MultiPolygon<F>, &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&MultiLineString<F>,    [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>, &MultiPolygon<F>, &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&MultiPolygon<F>,       [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>,                   &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&GeometryCollection<F>, [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>,                                           &Rect<F>, &Triangle<F>]);

// ┌──────────────────────────────┐
// │ Implementation for Geometry  │
// └──────────────────────────────┘

/// Euclidean distance implementation for every specific Geometry type to Geometry<T>.
macro_rules! impl_euclidean_distance_for_geometry_and_variant {
  ([$($target:ty),*]) => {
      $(
          impl<F: GeoFloat> Distance<F, $target, &Geometry<F>> for Euclidean {
              fn distance(&self, origin: $target, destination: &Geometry<F>) -> F {
                  match destination {
                      Geometry::Point(point) => self.distance(origin, point),
                      Geometry::Line(line) => self.distance(origin, line),
                      Geometry::LineString(line_string) => self.distance(origin, line_string),
                      Geometry::Polygon(polygon) => self.distance(origin, polygon),
                      Geometry::MultiPoint(multi_point) => self.distance(origin, multi_point),
                      Geometry::MultiLineString(multi_line_string) => self.distance(origin, multi_line_string),
                      Geometry::MultiPolygon(multi_polygon) => self.distance(origin, multi_polygon),
                      Geometry::GeometryCollection(geometry_collection) => self.distance(origin, geometry_collection),
                      Geometry::Rect(rect) => self.distance(origin, rect),
                      Geometry::Triangle(triangle) => self.distance(origin, triangle),
                  }
              }
          }
          symmetric_distance_impl!(GeoFloat, &Geometry<F>, $target);
      )*
  };
}

impl_euclidean_distance_for_geometry_and_variant!([&Point<F>, &MultiPoint<F>, &Line<F>, &LineString<F>, &MultiLineString<F>, &Polygon<F>, &MultiPolygon<F>, &Triangle<F>, &Rect<F>, &GeometryCollection<F>]);

impl<F: GeoFloat> Distance<F, &Geometry<F>, &Geometry<F>> for Euclidean {
    fn distance(&self, origin: &Geometry<F>, destination: &Geometry<F>) -> F {
        match origin {
            Geometry::Point(point) => self.distance(point, destination),
            Geometry::Line(line) => self.distance(line, destination),
            Geometry::LineString(line_string) => self.distance(line_string, destination),
            Geometry::Polygon(polygon) => self.distance(polygon, destination),
            Geometry::MultiPoint(multi_point) => self.distance(multi_point, destination),
            Geometry::MultiLineString(multi_line_string) => {
                self.distance(multi_line_string, destination)
            }
            Geometry::MultiPolygon(multi_polygon) => self.distance(multi_polygon, destination),
            Geometry::GeometryCollection(geometry_collection) => {
                self.distance(geometry_collection, destination)
            }
            Geometry::Rect(rect) => self.distance(rect, destination),
            Geometry::Triangle(triangle) => self.distance(triangle, destination),
        }
    }
}

// ┌───────────────────────────┐
// │ Implementations utilities │
// └───────────────────────────┘

/// Uses an R* tree and nearest-neighbour lookups to calculate minimum distances
// This is somewhat slow and memory-inefficient, but certainly better than quadratic time
fn nearest_neighbour_distance<F: GeoFloat>(geom1: &LineString<F>, geom2: &LineString<F>) -> F {
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // Return minimum distance between all geom a points and geom b lines, and all geom b points and geom a lines
    geom2
        .points()
        .fold(Bounded::max_value(), |acc: F, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(Euclidean.distance(nearest as &Line<F>, &point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            acc.min(Euclidean.distance(nearest as &Line<F>, &point))
        }))
}

fn ring_contains_coord<T: GeoNum>(ring: &LineString<T>, c: Coord<T>) -> bool {
    match coord_pos_relative_to_ring(c, ring) {
        CoordPos::Inside => true,
        CoordPos::OnBoundary | CoordPos::Outside => false,
    }
}

// ┌──────────────────────────────────────────────────────────┐
// │ Generic Trait Distance Extension - Direct Implementation │
// └──────────────────────────────────────────────────────────┘

use geo_traits::CoordTrait;
use geo_traits_ext::*;

/// Extension trait for generic geometry types to calculate distances directly
/// using Euclidean metric space without conversion overhead
/// Supports both same-type and cross-type distance calculations
pub trait DistanceExt<F: CoordFloat, Rhs = Self> {
    /// Calculate Euclidean distance using generic traits without conversion overhead
    fn distance_ext(&self, other: &Rhs) -> F;
}

// ┌──────────────────────────────────────────────────────────┐
// │ Generic trait macro implementations (following original) │
// └──────────────────────────────────────────────────────────┘

/// Generic trait version of polygon-like geometry distance implementation
/// Follows the same pattern as impl_euclidean_distance_for_polygonlike_geometry!
macro_rules! impl_distance_ext_for_polygonlike_geometry_trait {
    ($polygonlike_trait:ident, $polygonlike_tag:ident) => {
        impl<F, P: $polygonlike_trait<T = F>> GenericDistanceTrait<F, $polygonlike_tag> for P
        where
            F: GeoFloat,
        {
            fn generic_distance_trait(&self, other: &Self) -> F {
                let poly1 = self.to_polygon();
                let poly2 = other.to_polygon();
                poly1.distance_ext(&poly2)
            }
        }
    };
}

/// Generic trait version of multi-geometry distance implementation  
/// Follows the same pattern as impl_euclidean_distance_for_iter_geometry!
macro_rules! impl_distance_ext_for_iter_geometry_trait {
    ($iter_trait:ident, $iter_tag:ident, $member_method:ident) => {
        impl<F, I: $iter_trait<T = F>> GenericDistanceTrait<F, $iter_tag> for I
        where
            F: GeoFloat,
        {
            fn generic_distance_trait(&self, other: &Self) -> F {
                let mut min_dist: F = Float::max_value();
                for member1 in self.$member_method() {
                    for member2 in other.$member_method() {
                        let dist = member1.distance_ext(&member2);
                        min_dist = min_dist.min(dist);
                    }
                }
                if min_dist == Float::max_value() {
                    F::zero()
                } else {
                    min_dist
                }
            }
        }
    };
}

// ┌──────────────────────────────────────────────────────────┐
// │ Helper functions for generic trait operations              │
// └──────────────────────────────────────────────────────────┘

// Helper function for point distance using generic trait methods
fn point_distance_generic<F, P1, P2>(p1: &P1, p2: &P2) -> F
where
    F: CoordFloat,
    P1: PointTraitExt<T = F>,
    P2: PointTraitExt<T = F>,
{
    if let (Some(c1), Some(c2)) = (p1.coord(), p2.coord()) {
        let delta_x = c1.x() - c2.x();
        let delta_y = c1.y() - c2.y();
        delta_x.hypot(delta_y)
    } else {
        F::zero()
    }
}

// Helper for line segment distance using generic trait methods
fn line_segment_distance_generic<F, C, L>(coord: &C, line: &L) -> F
where
    F: CoordFloat,
    C: CoordTrait<T = F>,
    L: LineTraitExt<T = F>,
{
    let px = coord.x();
    let py = coord.y();
    let start = line.start_coord();
    let end = line.end_coord();
    let dx = end.x - start.x;
    let dy = end.y - start.y;

    if dx == F::zero() && dy == F::zero() {
        let delta_x = px - start.x;
        let delta_y = py - start.y;
        return delta_x.hypot(delta_y);
    }

    let t = ((px - start.x) * dx + (py - start.y) * dy) / (dx * dx + dy * dy);
    let t = t.max(F::zero()).min(F::one());

    let nearest_x = start.x + t * dx;
    let nearest_y = start.y + t * dy;
    let delta_x = px - nearest_x;
    let delta_y = py - nearest_y;
    delta_x.hypot(delta_y)
}

// ┌──────────────────────────────────────────────────────────┐
// │ Macros for symmetric distance implementations            │
// └──────────────────────────────────────────────────────────┘

// Macro for generating symmetric distance implementations
macro_rules! symmetric_distance_generic_impl {
    ($func_name_ab:ident, $func_name_ba:ident, $trait_a:ident, $trait_b:ident) => {
        pub fn $func_name_ba<F, A, B>(b: &B, a: &A) -> F
        where
            F: GeoFloat,
            A: $trait_a<T = F>,
            B: $trait_b<T = F>,
        {
            $func_name_ab(a, b)
        }
    };
}

// ┌────────────────────────────────────────────────────────────┐
// │ Cross-type distance functions (direct, no conversion)      │
// └────────────────────────────────────────────────────────────┘

// Point to LineString distance (direct, no conversion)
pub fn distance_point_to_linestring_generic<F, P, LS>(point: &P, linestring: &LS) -> F
where
    F: CoordFloat,
    P: PointTraitExt<T = F>,
    LS: LineStringTraitExt<T = F>,
{
    if let Some(coord) = point.coord() {
        linestring
            .lines()
            .map(|line| line_segment_distance_generic(&coord, &line))
            .fold(Float::max_value(), |acc, dist| acc.min(dist))
    } else {
        F::zero()
    }
}

// Point to Polygon distance (direct, no conversion)
pub fn distance_point_to_polygon_generic<F, P, Poly>(point: &P, polygon: &Poly) -> F
where
    F: GeoFloat,
    P: PointTraitExt<T = F>,
    Poly: PolygonTraitExt<T = F>,
{
    if let (Some(coord), Some(exterior)) = (point.coord(), polygon.exterior_ext()) {
        exterior
            .lines()
            .map(|line| line_segment_distance_generic(&coord, &line))
            .fold(Float::max_value(), |acc, dist| acc.min(dist))
    } else {
        F::zero()
    }
}

// LineString to Polygon distance (direct, no conversion)
pub fn distance_linestring_to_polygon_generic<F, LS, Poly>(linestring: &LS, polygon: &Poly) -> F
where
    F: GeoFloat,
    LS: LineStringTraitExt<T = F>,
    Poly: PolygonTraitExt<T = F>,
{
    if let Some(exterior) = polygon.exterior_ext() {
        let mut min_dist: F = Float::max_value();
        for line1 in linestring.lines() {
            for line2 in exterior.lines() {
                let d1 = line_segment_distance_generic(&line1.start_coord(), &line2);
                let d2 = line_segment_distance_generic(&line1.end_coord(), &line2);
                let d3 = line_segment_distance_generic(&line2.start_coord(), &line1);
                let d4 = line_segment_distance_generic(&line2.end_coord(), &line1);
                let line_dist = d1.min(d2).min(d3).min(d4);
                min_dist = min_dist.min(line_dist);
            }
        }
        if min_dist == Float::max_value() {
            F::zero()
        } else {
            min_dist
        }
    } else {
        F::zero()
    }
}

// Polygon to Polygon distance (direct, no conversion)
pub fn distance_polygon_to_polygon_generic<F, P1, P2>(polygon1: &P1, polygon2: &P2) -> F
where
    F: GeoFloat,
    P1: PolygonTraitExt<T = F>,
    P2: PolygonTraitExt<T = F>,
{
    if let (Some(ext1), Some(ext2)) = (polygon1.exterior_ext(), polygon2.exterior_ext()) {
        let mut min_dist: F = Float::max_value();
        for line1 in ext1.lines() {
            for line2 in ext2.lines() {
                let d1 = line_segment_distance_generic(&line1.start_coord(), &line2);
                let d2 = line_segment_distance_generic(&line1.end_coord(), &line2);
                let d3 = line_segment_distance_generic(&line2.start_coord(), &line1);
                let d4 = line_segment_distance_generic(&line2.end_coord(), &line1);
                let line_dist = d1.min(d2).min(d3).min(d4);
                min_dist = min_dist.min(line_dist);
            }
        }
        if min_dist == Float::max_value() {
            F::zero()
        } else {
            min_dist
        }
    } else {
        F::zero()
    }
}

// ┌────────────────────────────────────────────────────────────┐
// │ Generate symmetric functions using macros                  │
// └────────────────────────────────────────────────────────────┘

// Generate symmetric functions
symmetric_distance_generic_impl!(
    distance_point_to_linestring_generic,
    distance_linestring_to_point_generic,
    PointTraitExt,
    LineStringTraitExt
);

symmetric_distance_generic_impl!(
    distance_point_to_polygon_generic,
    distance_polygon_to_point_generic,
    PointTraitExt,
    PolygonTraitExt
);

symmetric_distance_generic_impl!(
    distance_linestring_to_polygon_generic,
    distance_polygon_to_linestring_generic,
    LineStringTraitExt,
    PolygonTraitExt
);

// ┌────────────────────────────────────────────────────────────┐
// │ Cross-type DistanceExt macro implementations               │
// └────────────────────────────────────────────────────────────┘

// ┌────────────────────────────────────────────────────────────┐
// │ DistanceExt trait implementation using type-tag pattern   │
// └────────────────────────────────────────────────────────────┘

// Implementation of DistanceExt for same-type generic trait geometries using the type-tag pattern
impl<F, G> DistanceExt<F> for G
where
    F: GeoFloat,
    G: GeoTraitExtWithTypeTag + GenericDistanceTrait<F, G::Tag>,
{
    fn distance_ext(&self, other: &G) -> F {
        self.generic_distance_trait(other)
    }
}

// Note: Cross-type distance support is implemented via the GeometryTag delegation pattern
// in the GenericDistanceTrait implementation above. This approach is different from the
// original Distance trait macro pattern due to Rust's coherence rules:
//
// Original Distance trait: impl Distance<F, A, B> for Euclidean
//   - Multiple implementations don't conflict because they're all for the same type (Euclidean)
//   - Can use macros to generate impl Distance<F, Point, LineString> for Euclidean, etc.
//
// DistanceExt trait: impl DistanceExt<F, B> for A
//   - Would conflict with blanket impl DistanceExt<F> for G when A == B
//   - Rust's orphan rule prevents having both blanket and specific implementations
//
// Solution: Use GeometryTraitExt runtime dispatch to handle all cross-type combinations
// This provides the same functionality while being compatible with Rust's type system.

// ┌────────────────────────────────────────────────────────────┐
// │ Internal trait for direct distance calculations            │
// └────────────────────────────────────────────────────────────┘

// Internal trait for direct distance calculations without conversion
trait GenericDistanceTrait<F, GT: GeoTypeTag>
where
    F: GeoFloat,
{
    fn generic_distance_trait(&self, other: &Self) -> F;
}

// ┌────────────────────────────────────────────────────────────┐
// │ Implementations for Point (generic traits)                │
// └────────────────────────────────────────────────────────────┘

// Point-to-Point direct distance implementation
impl<F, P: PointTraitExt<T = F>> GenericDistanceTrait<F, PointTag> for P
where
    F: GeoFloat,
{
    fn generic_distance_trait(&self, other: &Self) -> F {
        point_distance_generic(self, other)
    }
}

// ┌────────────────────────────────────────────────────────────┐
// │ Implementations for LineString (generic traits)           │
// └────────────────────────────────────────────────────────────┘

// LineString-to-LineString direct distance implementation
impl<F, LS: LineStringTraitExt<T = F>> GenericDistanceTrait<F, LineStringTag> for LS
where
    F: GeoFloat,
{
    fn generic_distance_trait(&self, other: &Self) -> F {
        let mut min_dist: F = Float::max_value();
        for line1 in self.lines() {
            for line2 in other.lines() {
                // Line-to-line distance using endpoints
                let d1 = line_segment_distance_generic(&line1.start_coord(), &line2);
                let d2 = line_segment_distance_generic(&line1.end_coord(), &line2);
                let d3 = line_segment_distance_generic(&line2.start_coord(), &line1);
                let d4 = line_segment_distance_generic(&line2.end_coord(), &line1);
                let line_dist = d1.min(d2).min(d3).min(d4);
                min_dist = min_dist.min(line_dist);
            }
        }
        if min_dist == Float::max_value() {
            F::zero()
        } else {
            min_dist
        }
    }
}

// ┌────────────────────────────────────────────────────────────┐
// │ Implementations for Polygon (generic traits)              │
// └────────────────────────────────────────────────────────────┘

// Polygon-to-Polygon direct distance implementation
impl<F, P: PolygonTraitExt<T = F>> GenericDistanceTrait<F, PolygonTag> for P
where
    F: GeoFloat,
{
    fn generic_distance_trait(&self, other: &Self) -> F {
        if let (Some(ext1), Some(ext2)) = (self.exterior_ext(), other.exterior_ext()) {
            ext1.distance_ext(&ext2)
        } else {
            F::zero()
        }
    }
}

// ┌────────────────────────────────────────────────────────────┐
// │ Implementations for Rect and Triangle (generic traits)     │
// └────────────────────────────────────────────────────────────┘

impl_distance_ext_for_polygonlike_geometry_trait!(TriangleTraitExt, TriangleTag);
impl_distance_ext_for_polygonlike_geometry_trait!(RectTraitExt, RectTag);

// ┌────────────────────────────────────────────────────────────┐
// │ Implementations for multi-geometry types (generic traits)  │
// └────────────────────────────────────────────────────────────┘

impl_distance_ext_for_iter_geometry_trait!(MultiPointTraitExt, MultiPointTag, points_ext);
impl_distance_ext_for_iter_geometry_trait!(
    MultiLineStringTraitExt,
    MultiLineStringTag,
    line_strings_ext
);
impl_distance_ext_for_iter_geometry_trait!(MultiPolygonTraitExt, MultiPolygonTag, polygons_ext);

// ┌────────────────────────────────────────────────────────────┐
// │ Implementation for Geometry (generic traits)               │
// └────────────────────────────────────────────────────────────┘

// Generic trait distance implementation for Geometry dispatch.
impl<F, G: GeometryTraitExt<T = F>> GenericDistanceTrait<F, GeometryTag> for G
where
    F: GeoFloat,
{
    fn generic_distance_trait(&self, other: &Self) -> F {
        use geo_traits_ext::GeometryTypeExt;

        match (self.as_type_ext(), other.as_type_ext()) {
            // Same-type combinations
            (GeometryTypeExt::Point(p1), GeometryTypeExt::Point(p2)) => p1.distance_ext(p2),
            (GeometryTypeExt::LineString(ls1), GeometryTypeExt::LineString(ls2)) => {
                ls1.distance_ext(ls2)
            }
            (GeometryTypeExt::Polygon(poly1), GeometryTypeExt::Polygon(poly2)) => {
                poly1.distance_ext(poly2)
            }
            (GeometryTypeExt::MultiPoint(mp1), GeometryTypeExt::MultiPoint(mp2)) => {
                mp1.distance_ext(mp2)
            }
            (GeometryTypeExt::MultiLineString(mls1), GeometryTypeExt::MultiLineString(mls2)) => {
                mls1.distance_ext(mls2)
            }
            (GeometryTypeExt::MultiPolygon(mp1), GeometryTypeExt::MultiPolygon(mp2)) => {
                mp1.distance_ext(mp2)
            }
            (GeometryTypeExt::Rect(rect1), GeometryTypeExt::Rect(rect2)) => {
                rect1.distance_ext(rect2)
            }
            (GeometryTypeExt::Triangle(tri1), GeometryTypeExt::Triangle(tri2)) => {
                tri1.distance_ext(tri2)
            }

            // Cross-type combinations using helper functions directly
            (GeometryTypeExt::Point(point), GeometryTypeExt::LineString(linestring)) => {
                distance_point_to_linestring_generic(point, linestring)
            }
            (GeometryTypeExt::LineString(linestring), GeometryTypeExt::Point(point)) => {
                distance_linestring_to_point_generic(linestring, point)
            }
            (GeometryTypeExt::Point(point), GeometryTypeExt::Polygon(polygon)) => {
                distance_point_to_polygon_generic(point, polygon)
            }
            (GeometryTypeExt::Polygon(polygon), GeometryTypeExt::Point(point)) => {
                distance_polygon_to_point_generic(polygon, point)
            }
            (GeometryTypeExt::LineString(linestring), GeometryTypeExt::Polygon(polygon)) => {
                distance_linestring_to_polygon_generic(linestring, polygon)
            }
            (GeometryTypeExt::Polygon(polygon), GeometryTypeExt::LineString(linestring)) => {
                distance_polygon_to_linestring_generic(polygon, linestring)
            }

            // Cross-type combinations with Rect (convert to polygon)
            (GeometryTypeExt::Point(point), GeometryTypeExt::Rect(rect)) => {
                let poly = rect.to_polygon();
                distance_point_to_polygon_generic(point, &poly)
            }
            (GeometryTypeExt::Rect(rect), GeometryTypeExt::Point(point)) => {
                let poly = rect.to_polygon();
                distance_polygon_to_point_generic(&poly, point)
            }
            (GeometryTypeExt::LineString(linestring), GeometryTypeExt::Rect(rect)) => {
                let poly = rect.to_polygon();
                distance_linestring_to_polygon_generic(linestring, &poly)
            }
            (GeometryTypeExt::Rect(rect), GeometryTypeExt::LineString(linestring)) => {
                let poly = rect.to_polygon();
                distance_polygon_to_linestring_generic(&poly, linestring)
            }
            (GeometryTypeExt::Polygon(polygon), GeometryTypeExt::Rect(rect)) => {
                let poly = rect.to_polygon();
                distance_polygon_to_polygon_generic(polygon, &poly)
            }
            (GeometryTypeExt::Rect(rect), GeometryTypeExt::Polygon(polygon)) => {
                let poly = rect.to_polygon();
                distance_polygon_to_polygon_generic(&poly, polygon)
            }

            // Cross-type combinations with Triangle (convert to polygon)
            (GeometryTypeExt::Point(point), GeometryTypeExt::Triangle(tri)) => {
                let poly = tri.to_polygon();
                distance_point_to_polygon_generic(point, &poly)
            }
            (GeometryTypeExt::Triangle(tri), GeometryTypeExt::Point(point)) => {
                let poly = tri.to_polygon();
                distance_polygon_to_point_generic(&poly, point)
            }
            (GeometryTypeExt::LineString(linestring), GeometryTypeExt::Triangle(tri)) => {
                let poly = tri.to_polygon();
                distance_linestring_to_polygon_generic(linestring, &poly)
            }
            (GeometryTypeExt::Triangle(tri), GeometryTypeExt::LineString(linestring)) => {
                let poly = tri.to_polygon();
                distance_polygon_to_linestring_generic(&poly, linestring)
            }
            (GeometryTypeExt::Polygon(polygon), GeometryTypeExt::Triangle(tri)) => {
                let poly = tri.to_polygon();
                distance_polygon_to_polygon_generic(polygon, &poly)
            }
            (GeometryTypeExt::Triangle(tri), GeometryTypeExt::Polygon(polygon)) => {
                let poly = tri.to_polygon();
                distance_polygon_to_polygon_generic(&poly, polygon)
            }
            (GeometryTypeExt::Rect(rect), GeometryTypeExt::Triangle(tri)) => {
                let rect_poly = rect.to_polygon();
                let tri_poly = tri.to_polygon();
                distance_polygon_to_polygon_generic(&rect_poly, &tri_poly)
            }
            (GeometryTypeExt::Triangle(tri), GeometryTypeExt::Rect(rect)) => {
                let tri_poly = tri.to_polygon();
                let rect_poly = rect.to_polygon();
                distance_polygon_to_polygon_generic(&tri_poly, &rect_poly)
            }

            // Multi-geometry cross-type combinations using fold operations
            (GeometryTypeExt::Point(point), GeometryTypeExt::MultiPoint(mp)) => mp
                .points_ext()
                .map(|p| point_distance_generic(point, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPoint(mp), GeometryTypeExt::Point(point)) => mp
                .points_ext()
                .map(|p| point_distance_generic(&p, point))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::Point(point), GeometryTypeExt::MultiLineString(mls)) => mls
                .line_strings_ext()
                .map(|ls| distance_point_to_linestring_generic(point, &ls))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiLineString(mls), GeometryTypeExt::Point(point)) => mls
                .line_strings_ext()
                .map(|ls| distance_linestring_to_point_generic(&ls, point))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::Point(point), GeometryTypeExt::MultiPolygon(mp)) => mp
                .polygons_ext()
                .map(|p| distance_point_to_polygon_generic(point, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPolygon(mp), GeometryTypeExt::Point(point)) => mp
                .polygons_ext()
                .map(|p| distance_polygon_to_point_generic(&p, point))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),

            // LineString to multi-geometry combinations
            (GeometryTypeExt::LineString(ls), GeometryTypeExt::MultiPoint(mp)) => mp
                .points_ext()
                .map(|p| distance_linestring_to_point_generic(ls, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPoint(mp), GeometryTypeExt::LineString(ls)) => mp
                .points_ext()
                .map(|p| distance_point_to_linestring_generic(&p, ls))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::LineString(ls), GeometryTypeExt::MultiLineString(mls)) => {
                let mut min_dist: F = Float::max_value();
                for other_ls in mls.line_strings_ext() {
                    for line1 in ls.lines() {
                        for line2 in other_ls.lines() {
                            let d1 = line_segment_distance_generic(&line1.start_coord(), &line2);
                            let d2 = line_segment_distance_generic(&line1.end_coord(), &line2);
                            let d3 = line_segment_distance_generic(&line2.start_coord(), &line1);
                            let d4 = line_segment_distance_generic(&line2.end_coord(), &line1);
                            min_dist = min_dist.min(d1.min(d2).min(d3).min(d4));
                        }
                    }
                }
                if min_dist == Float::max_value() {
                    F::zero()
                } else {
                    min_dist
                }
            }
            (GeometryTypeExt::MultiLineString(mls), GeometryTypeExt::LineString(ls)) => {
                let mut min_dist: F = Float::max_value();
                for other_ls in mls.line_strings_ext() {
                    for line1 in other_ls.lines() {
                        for line2 in ls.lines() {
                            let d1 = line_segment_distance_generic(&line1.start_coord(), &line2);
                            let d2 = line_segment_distance_generic(&line1.end_coord(), &line2);
                            let d3 = line_segment_distance_generic(&line2.start_coord(), &line1);
                            let d4 = line_segment_distance_generic(&line2.end_coord(), &line1);
                            min_dist = min_dist.min(d1.min(d2).min(d3).min(d4));
                        }
                    }
                }
                if min_dist == Float::max_value() {
                    F::zero()
                } else {
                    min_dist
                }
            }
            (GeometryTypeExt::LineString(ls), GeometryTypeExt::MultiPolygon(mp)) => mp
                .polygons_ext()
                .map(|p| distance_linestring_to_polygon_generic(ls, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPolygon(mp), GeometryTypeExt::LineString(ls)) => mp
                .polygons_ext()
                .map(|p| distance_polygon_to_linestring_generic(&p, ls))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),

            // Polygon to multi-geometry combinations
            (GeometryTypeExt::Polygon(poly), GeometryTypeExt::MultiPoint(mp)) => mp
                .points_ext()
                .map(|p| distance_polygon_to_point_generic(poly, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPoint(mp), GeometryTypeExt::Polygon(poly)) => mp
                .points_ext()
                .map(|p| distance_point_to_polygon_generic(&p, poly))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::Polygon(poly), GeometryTypeExt::MultiLineString(mls)) => mls
                .line_strings_ext()
                .map(|ls| distance_polygon_to_linestring_generic(poly, &ls))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiLineString(mls), GeometryTypeExt::Polygon(poly)) => mls
                .line_strings_ext()
                .map(|ls| distance_linestring_to_polygon_generic(&ls, poly))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::Polygon(poly), GeometryTypeExt::MultiPolygon(mp)) => mp
                .polygons_ext()
                .map(|p| distance_polygon_to_polygon_generic(poly, &p))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),
            (GeometryTypeExt::MultiPolygon(mp), GeometryTypeExt::Polygon(poly)) => mp
                .polygons_ext()
                .map(|p| distance_polygon_to_polygon_generic(&p, poly))
                .fold(Float::max_value(), |acc, dist| acc.min(dist)),

            // For unsupported combinations, return zero
            _ => F::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orient::{Direction, Orient};
    use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
    use geo_types::{coord, polygon, private_utils::line_segment_distance};

    // ┌────────────────────────────────────────────────────────────────┐
    // │ Tests for original Distance trait (concrete implementations)   │
    // └────────────────────────────────────────────────────────────────┘

    mod original_distance_tests {
        use super::*;

        #[test]
        fn line_segment_distance_test() {
            let o1 = Point::new(8.0, 0.0);
            let o2 = Point::new(5.5, 0.0);
            let o3 = Point::new(5.0, 0.0);
            let o4 = Point::new(4.5, 1.5);

            let p1 = Point::new(7.2, 2.0);
            let p2 = Point::new(6.0, 1.0);

            let dist = line_segment_distance(o1, p1, p2);
            let dist2 = line_segment_distance(o2, p1, p2);
            let dist3 = line_segment_distance(o3, p1, p2);
            let dist4 = line_segment_distance(o4, p1, p2);
            // Results agree with Shapely
            assert_relative_eq!(dist, 2.0485900789263356);
            assert_relative_eq!(dist2, 1.118033988749895);
            assert_relative_eq!(dist3, std::f64::consts::SQRT_2); // workaround clippy::correctness error approx_constant (1.4142135623730951)
            assert_relative_eq!(dist4, 1.5811388300841898);
            // Point is on the line
            let zero_dist = line_segment_distance(p1, p1, p2);
            assert_relative_eq!(zero_dist, 0.0);
        }
        #[test]
        // Point to Polygon, outside point
        fn point_polygon_distance_outside_test() {
            // an octagon
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
                (5., 1.),
            ];
            let ls = LineString::from(points);
            let poly = Polygon::new(ls, vec![]);
            // A Random point outside the octagon
            let p = Point::new(2.5, 0.5);
            let dist = Euclidean.distance(&p, &poly);
            assert_relative_eq!(dist, 2.1213203435596424);
        }
        #[test]
        // Point to Polygon, inside point
        fn point_polygon_distance_inside_test() {
            // an octagon
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
                (5., 1.),
            ];
            let ls = LineString::from(points);
            let poly = Polygon::new(ls, vec![]);
            // A Random point inside the octagon
            let p = Point::new(5.5, 2.1);
            let dist = Euclidean.distance(&p, &poly);
            assert_relative_eq!(dist, 0.0);
        }
        #[test]
        // Point to Polygon, on boundary
        fn point_polygon_distance_boundary_test() {
            // an octagon
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
                (5., 1.),
            ];
            let ls = LineString::from(points);
            let poly = Polygon::new(ls, vec![]);
            // A point on the octagon
            let p = Point::new(5.0, 1.0);
            let dist = Euclidean.distance(&p, &poly);
            assert_relative_eq!(dist, 0.0);
        }
        #[test]
        // Point to Polygon, on boundary
        fn point_polygon_boundary_test2() {
            let exterior = LineString::from(vec![
                (0., 0.),
                (0., 0.0004),
                (0.0004, 0.0004),
                (0.0004, 0.),
                (0., 0.),
            ]);

            let poly = Polygon::new(exterior, vec![]);
            let bugged_point = Point::new(0.0001, 0.);
            assert_relative_eq!(Euclidean.distance(&poly, &bugged_point), 0.);
        }
        #[test]
        // Point to Polygon, empty Polygon
        fn point_polygon_empty_test() {
            // an empty Polygon
            let points = vec![];
            let ls = LineString::new(points);
            let poly = Polygon::new(ls, vec![]);
            // A point on the octagon
            let p = Point::new(2.5, 0.5);
            let dist = Euclidean.distance(&p, &poly);
            assert_relative_eq!(dist, 0.0);
        }
        #[test]
        // Point to Polygon with an interior ring
        fn point_polygon_interior_cutout_test() {
            // an octagon
            let ext_points = vec![
                (4., 1.),
                (5., 2.),
                (5., 3.),
                (4., 4.),
                (3., 4.),
                (2., 3.),
                (2., 2.),
                (3., 1.),
                (4., 1.),
            ];
            // cut out a triangle inside octagon
            let int_points = vec![(3.5, 3.5), (4.4, 1.5), (2.6, 1.5), (3.5, 3.5)];
            let ls_ext = LineString::from(ext_points);
            let ls_int = LineString::from(int_points);
            let poly = Polygon::new(ls_ext, vec![ls_int]);
            // A point inside the cutout triangle
            let p = Point::new(3.5, 2.5);
            let dist = Euclidean.distance(&p, &poly);

            // 0.41036467732879783 <-- Shapely
            assert_relative_eq!(dist, 0.41036467732879767);
        }

        #[test]
        fn line_distance_multipolygon_do_not_intersect_test() {
            // checks that the distance from the multipolygon
            // is equal to the distance from the closest polygon
            // taken in isolation, whatever that distance is
            let ls1 = LineString::from(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (5.0, 15.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]);
            let ls2 = LineString::from(vec![
                (0.0, 30.0),
                (0.0, 25.0),
                (10.0, 25.0),
                (10.0, 30.0),
                (0.0, 30.0),
            ]);
            let ls3 = LineString::from(vec![
                (15.0, 30.0),
                (15.0, 25.0),
                (20.0, 25.0),
                (20.0, 30.0),
                (15.0, 30.0),
            ]);
            let pol1 = Polygon::new(ls1, vec![]);
            let pol2 = Polygon::new(ls2, vec![]);
            let pol3 = Polygon::new(ls3, vec![]);
            let mp = MultiPolygon::new(vec![pol1.clone(), pol2, pol3]);
            let pnt1 = Point::new(0.0, 15.0);
            let pnt2 = Point::new(10.0, 20.0);
            let ln = Line::new(pnt1.0, pnt2.0);
            let dist_mp_ln = Euclidean.distance(&ln, &mp);
            let dist_pol1_ln = Euclidean.distance(&ln, &pol1);
            assert_relative_eq!(dist_mp_ln, dist_pol1_ln);
        }

        #[test]
        fn point_distance_multipolygon_test() {
            let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
            let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
            let p1 = Polygon::new(ls1, vec![]);
            let p2 = Polygon::new(ls2, vec![]);
            let mp = MultiPolygon::new(vec![p1, p2]);
            let p = Point::new(50.0, 50.0);
            assert_relative_eq!(Euclidean.distance(&p, &mp), 60.959002616512684);
        }
        #[test]
        // Point to LineString
        fn point_linestring_distance_test() {
            // like an octagon, but missing the lowest horizontal segment
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
            ];
            let ls = LineString::from(points);
            // A Random point "inside" the LineString
            let p = Point::new(5.5, 2.1);
            let dist = Euclidean.distance(&p, &ls);
            assert_relative_eq!(dist, 1.1313708498984762);
        }
        #[test]
        // Point to LineString, point lies on the LineString
        fn point_linestring_contains_test() {
            // like an octagon, but missing the lowest horizontal segment
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
            ];
            let ls = LineString::from(points);
            // A point which lies on the LineString
            let p = Point::new(5.0, 4.0);
            let dist = Euclidean.distance(&p, &ls);
            assert_relative_eq!(dist, 0.0);
        }
        #[test]
        // Point to LineString, closed triangle
        fn point_linestring_triangle_test() {
            let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
            let ls = LineString::from(points);
            let p = Point::new(3.5, 2.5);
            let dist = Euclidean.distance(&p, &ls);
            assert_relative_eq!(dist, 0.5);
        }
        #[test]
        // Point to LineString, empty LineString
        fn point_linestring_empty_test() {
            let points = vec![];
            let ls = LineString::new(points);
            let p = Point::new(5.0, 4.0);
            let dist = Euclidean.distance(&p, &ls);
            assert_relative_eq!(dist, 0.0);
        }
        #[test]
        fn distance_multilinestring_test() {
            let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
            let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
            let mls = MultiLineString::new(vec![v1, v2]);
            let p = Point::new(50.0, 50.0);
            assert_relative_eq!(Euclidean.distance(&p, &mls), 63.25345840347388);
        }
        #[test]
        fn distance1_test() {
            assert_relative_eq!(
                Euclidean.distance(&Point::new(0., 0.), &Point::new(1., 0.)),
                1.
            );
        }
        #[test]
        fn distance2_test() {
            let dist =
                Euclidean.distance(&Point::new(-72.1235, 42.3521), &Point::new(72.1260, 70.612));
            assert_relative_eq!(dist, 146.99163308930207);
        }
        #[test]
        fn distance_multipoint_test() {
            let v = vec![
                Point::new(0.0, 10.0),
                Point::new(1.0, 1.0),
                Point::new(10.0, 0.0),
                Point::new(1.0, -1.0),
                Point::new(0.0, -10.0),
                Point::new(-1.0, -1.0),
                Point::new(-10.0, 0.0),
                Point::new(-1.0, 1.0),
                Point::new(0.0, 10.0),
            ];
            let mp = MultiPoint::new(v);
            let p = Point::new(50.0, 50.0);
            assert_relative_eq!(Euclidean.distance(&p, &mp), 64.03124237432849)
        }
        #[test]
        fn distance_line_test() {
            let line0 = Line::from([(0., 0.), (5., 0.)]);
            let p0 = Point::new(2., 3.);
            let p1 = Point::new(3., 0.);
            let p2 = Point::new(6., 0.);
            assert_relative_eq!(Euclidean.distance(&line0, &p0), 3.);
            assert_relative_eq!(Euclidean.distance(&p0, &line0), 3.);

            assert_relative_eq!(Euclidean.distance(&line0, &p1), 0.);
            assert_relative_eq!(Euclidean.distance(&p1, &line0), 0.);

            assert_relative_eq!(Euclidean.distance(&line0, &p2), 1.);
            assert_relative_eq!(Euclidean.distance(&p2, &line0), 1.);
        }
        #[test]
        fn distance_line_line_test() {
            let line0 = Line::from([(0., 0.), (5., 0.)]);
            let line1 = Line::from([(2., 1.), (7., 2.)]);
            assert_relative_eq!(Euclidean.distance(&line0, &line1), 1.);
            assert_relative_eq!(Euclidean.distance(&line1, &line0), 1.);
        }
        #[test]
        // See https://github.com/georust/geo/issues/476
        fn distance_line_polygon_test() {
            let line = Line::new(
                coord! {
                    x: -0.17084137691985102,
                    y: 0.8748085493016657,
                },
                coord! {
                    x: -0.17084137691985102,
                    y: 0.09858870312437906,
                },
            );
            let poly: Polygon<f64> = polygon![
                coord! {
                    x: -0.10781391405721802,
                    y: -0.15433610862574643,
                },
                coord! {
                    x: -0.7855276236615211,
                    y: 0.23694208404779793,
                },
                coord! {
                    x: -0.7855276236615214,
                    y: -0.5456143012992907,
                },
                coord! {
                    x: -0.10781391405721802,
                    y: -0.15433610862574643,
                },
            ];
            assert_eq!(Euclidean.distance(&line, &poly), 0.18752558079168907);
        }
        #[test]
        // test edge-vertex minimum distance
        fn test_minimum_polygon_distance() {
            let points_raw = [
                (126., 232.),
                (126., 212.),
                (112., 202.),
                (97., 204.),
                (87., 215.),
                (87., 232.),
                (100., 246.),
                (118., 247.),
            ];
            let points = points_raw
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly1 = Polygon::new(LineString::from(points), vec![]);

            let points_raw_2 = [
                (188., 231.),
                (189., 207.),
                (174., 196.),
                (164., 196.),
                (147., 220.),
                (158., 242.),
                (177., 242.),
            ];
            let points2 = points_raw_2
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly2 = Polygon::new(LineString::from(points2), vec![]);
            let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
            assert_relative_eq!(dist, 21.0);
        }
        #[test]
        // test vertex-vertex minimum distance
        fn test_minimum_polygon_distance_2() {
            let points_raw = [
                (118., 200.),
                (153., 179.),
                (106., 155.),
                (88., 190.),
                (118., 200.),
            ];
            let points = points_raw
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly1 = Polygon::new(LineString::from(points), vec![]);

            let points_raw_2 = [
                (242., 186.),
                (260., 146.),
                (182., 175.),
                (216., 193.),
                (242., 186.),
            ];
            let points2 = points_raw_2
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly2 = Polygon::new(LineString::from(points2), vec![]);
            let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
            assert_relative_eq!(dist, 29.274562336608895);
        }
        #[test]
        // test edge-edge minimum distance
        fn test_minimum_polygon_distance_3() {
            let points_raw = [
                (182., 182.),
                (182., 168.),
                (138., 160.),
                (136., 193.),
                (182., 182.),
            ];
            let points = points_raw
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly1 = Polygon::new(LineString::from(points), vec![]);

            let points_raw_2 = [
                (232., 196.),
                (234., 150.),
                (194., 165.),
                (194., 191.),
                (232., 196.),
            ];
            let points2 = points_raw_2
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
            let poly2 = Polygon::new(LineString::from(points2), vec![]);
            let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
            assert_relative_eq!(dist, 12.0);
        }
        #[test]
        fn test_large_polygon_distance() {
            let ls = geo_test_fixtures::norway_main::<f64>();
            let poly1 = Polygon::new(ls, vec![]);
            let vec2 = vec![
                (4.921875, 66.33750501996518),
                (3.69140625, 65.21989393613207),
                (6.15234375, 65.07213008560697),
                (4.921875, 66.33750501996518),
            ];
            let poly2 = Polygon::new(vec2.into(), vec![]);
            let distance = Euclidean.distance(&poly1, &poly2);
            // GEOS says 2.2864896295566055
            assert_relative_eq!(distance, 2.2864896295566055);
        }
        #[test]
        // A polygon inside another polygon's ring; they're disjoint in the DE-9IM sense:
        // FF2FF1212
        fn test_poly_in_ring() {
            let shell = geo_test_fixtures::shell::<f64>();
            let ring = geo_test_fixtures::ring::<f64>();
            let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
            // inside is "inside" outside's ring, but they are disjoint
            let outside = Polygon::new(shell, vec![ring]);
            let inside = Polygon::new(poly_in_ring, vec![]);
            assert_relative_eq!(Euclidean.distance(&outside, &inside), 5.992772737231033);
        }
        #[test]
        // two ring LineStrings; one encloses the other but they neither touch nor intersect
        fn test_linestring_distance() {
            let ring = geo_test_fixtures::ring::<f64>();
            let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
            assert_relative_eq!(Euclidean.distance(&ring, &poly_in_ring), 5.992772737231033);
        }
        #[test]
        // Line-Polygon test: closest point on Polygon is NOT nearest to a Line end-point
        fn test_line_polygon_simple() {
            let line = Line::from([(0.0, 0.0), (0.0, 3.0)]);
            let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
            let poly = Polygon::new(v.into(), vec![]);
            assert_relative_eq!(Euclidean.distance(&line, &poly), 0.25);
        }
        #[test]
        // Line-Polygon test: Line intersects Polygon
        fn test_line_polygon_intersects() {
            let line = Line::from([(0.5, 0.0), (0.0, 3.0)]);
            let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
            let poly = Polygon::new(v.into(), vec![]);
            assert_relative_eq!(Euclidean.distance(&line, &poly), 0.0);
        }
        #[test]
        // Line-Polygon test: Line contained by interior ring
        fn test_line_polygon_inside_ring() {
            let line = Line::from([(4.4, 1.5), (4.45, 1.5)]);
            let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.0), (5.0, 1.0)];
            let v2 = vec![(4.5, 1.2), (4.5, 1.8), (3.5, 1.2), (4.5, 1.2)];
            let poly = Polygon::new(v.into(), vec![v2.into()]);
            assert_relative_eq!(Euclidean.distance(&line, &poly), 0.04999999999999982);
        }
        #[test]
        // LineString-Line test
        fn test_linestring_line_distance() {
            let line = Line::from([(0.0, 0.0), (0.0, 2.0)]);
            let ls: LineString<_> = vec![(3.0, 0.0), (1.0, 1.0), (3.0, 2.0)].into();
            assert_relative_eq!(Euclidean.distance(&ls, &line), 1.0);
        }

        #[test]
        // Triangle-Point test: point on vertex
        fn test_triangle_point_on_vertex_distance() {
            let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
            let point = Point::new(0.0, 0.0);
            assert_relative_eq!(Euclidean.distance(&triangle, &point), 0.0);
        }

        #[test]
        // Triangle-Point test: point on edge
        fn test_triangle_point_on_edge_distance() {
            let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
            let point = Point::new(1.5, 0.0);
            assert_relative_eq!(Euclidean.distance(&triangle, &point), 0.0);
        }

        #[test]
        // Triangle-Point test
        fn test_triangle_point_distance() {
            let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
            let point = Point::new(2.0, 3.0);
            assert_relative_eq!(Euclidean.distance(&triangle, &point), 1.0);
        }

        #[test]
        // Triangle-Point test: point within triangle
        fn test_triangle_point_inside_distance() {
            let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
            let point = Point::new(1.0, 0.5);
            assert_relative_eq!(Euclidean.distance(&triangle, &point), 0.0);
        }

        #[test]
        fn convex_and_nearest_neighbour_comparison() {
            let ls1: LineString<f64> = vec![
                Coord::from((57.39453770777941, 307.60533608924663)),
                Coord::from((67.1800355576469, 309.6654408997451)),
                Coord::from((84.89693692793338, 225.5101593908847)),
                Coord::from((75.1114390780659, 223.45005458038628)),
                Coord::from((57.39453770777941, 307.60533608924663)),
            ]
            .into();
            let first_polygon: Polygon<f64> = Polygon::new(ls1, vec![]);
            let ls2: LineString<f64> = vec![
                Coord::from((138.11769866645008, -45.75134112915392)),
                Coord::from((130.50230476949187, -39.270154833870336)),
                Coord::from((184.94426964987397, 24.699153900578573)),
                Coord::from((192.55966354683218, 18.217967605294987)),
                Coord::from((138.11769866645008, -45.75134112915392)),
            ]
            .into();
            let second_polygon = Polygon::new(ls2, vec![]);

            assert_relative_eq!(
                Euclidean.distance(&first_polygon, &second_polygon),
                224.35357967013238
            );
        }
        #[test]
        fn fast_path_regression() {
            // this test will fail if the fast path algorithm is reintroduced without being fixed
            let p1 = polygon!(
                (x: 0_f64, y: 0_f64),
                (x: 300_f64, y: 0_f64),
                (x: 300_f64, y: 100_f64),
                (x: 0_f64, y: 100_f64),
            )
            .orient(Direction::Default);
            let p2 = polygon!(
                (x: 100_f64, y: 150_f64),
                (x: 150_f64, y: 200_f64),
                (x: 50_f64, y: 200_f64),
            )
            .orient(Direction::Default);
            let p3 = polygon!(
                (x: 0_f64, y: 0_f64),
                (x: 300_f64, y: 0_f64),
                (x: 300_f64, y: 100_f64),
                (x: 0_f64, y: 100_f64),
            )
            .orient(Direction::Reversed);
            let p4 = polygon!(
                (x: 100_f64, y: 150_f64),
                (x: 150_f64, y: 200_f64),
                (x: 50_f64, y: 200_f64),
            )
            .orient(Direction::Reversed);
            assert_eq!(Euclidean.distance(&p1, &p2), 50.0f64);
            assert_eq!(Euclidean.distance(&p3, &p4), 50.0f64);
            assert_eq!(Euclidean.distance(&p1, &p4), 50.0f64);
            assert_eq!(Euclidean.distance(&p2, &p3), 50.0f64);
        }
        #[test]
        fn rect_to_polygon_distance_test() {
            // Test that Rect to Polygon distance works
            let rect = Rect::new((0.0, 0.0), (2.0, 2.0));
            let poly_points = vec![(3., 0.), (5., 0.), (5., 2.), (3., 2.), (3., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);

            // Test both directions
            let dist1 = Euclidean.distance(&rect, &poly);
            let dist2 = Euclidean.distance(&poly, &rect);

            assert_relative_eq!(dist1, 1.0);
            assert_relative_eq!(dist2, 1.0);
            assert_relative_eq!(dist1, dist2); // Verify symmetry
        }

        #[test]
        fn all_types_geometry_collection_test() {
            let p = Point::new(0.0, 0.0);
            let line = Line::from([(-1.0, -1.0), (-2.0, -2.0)]);
            let ls = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0)]);
            let poly = Polygon::new(
                LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]),
                vec![],
            );
            let tri = Triangle::from([(0.0, 0.0), (1.0, 10.0), (2.0, 0.0)]);
            let rect = Rect::new((0.0, 0.0), (-1.0, -1.0));

            let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
            let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
            let p1 = Polygon::new(ls1, vec![]);
            let p2 = Polygon::new(ls2, vec![]);
            let mpoly = MultiPolygon::new(vec![p1, p2]);

            let v = vec![
                Point::new(0.0, 10.0),
                Point::new(1.0, 1.0),
                Point::new(10.0, 0.0),
                Point::new(1.0, -1.0),
                Point::new(0.0, -10.0),
                Point::new(-1.0, -1.0),
                Point::new(-10.0, 0.0),
                Point::new(-1.0, 1.0),
                Point::new(0.0, 10.0),
            ];
            let mpoint = MultiPoint::new(v);

            let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
            let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
            let mls = MultiLineString::new(vec![v1, v2]);

            let gc = GeometryCollection(vec![
                Geometry::Point(p),
                Geometry::Line(line),
                Geometry::LineString(ls),
                Geometry::Polygon(poly),
                Geometry::MultiPoint(mpoint),
                Geometry::MultiLineString(mls),
                Geometry::MultiPolygon(mpoly),
                Geometry::Triangle(tri),
                Geometry::Rect(rect),
            ]);

            let test_p = Point::new(50., 50.);
            assert_relative_eq!(Euclidean.distance(&test_p, &gc), 60.959002616512684);

            let test_multipoint = MultiPoint::new(vec![test_p]);
            assert_relative_eq!(
                Euclidean.distance(&test_multipoint, &gc),
                60.959002616512684
            );

            let test_line = Line::from([(50., 50.), (60., 60.)]);
            assert_relative_eq!(Euclidean.distance(&test_line, &gc), 60.959002616512684);

            let test_ls = LineString::from(vec![(50., 50.), (60., 60.), (70., 70.)]);
            assert_relative_eq!(Euclidean.distance(&test_ls, &gc), 60.959002616512684);

            let test_mls = MultiLineString::new(vec![test_ls]);
            assert_relative_eq!(Euclidean.distance(&test_mls, &gc), 60.959002616512684);

            let test_poly = Polygon::new(
                LineString::from(vec![
                    (50., 50.),
                    (60., 50.),
                    (60., 60.),
                    (55., 55.),
                    (50., 50.),
                ]),
                vec![],
            );
            assert_relative_eq!(Euclidean.distance(&test_poly, &gc), 60.959002616512684);

            let test_multipoly = MultiPolygon::new(vec![test_poly]);
            assert_relative_eq!(Euclidean.distance(&test_multipoly, &gc), 60.959002616512684);

            let test_tri = Triangle::from([(50., 50.), (60., 50.), (55., 55.)]);
            assert_relative_eq!(Euclidean.distance(&test_tri, &gc), 60.959002616512684);

            let test_rect = Rect::new(coord! { x: 50., y: 50. }, coord! { x: 60., y: 60. });
            assert_relative_eq!(Euclidean.distance(&test_rect, &gc), 60.959002616512684);

            let test_gc = GeometryCollection(vec![Geometry::Rect(test_rect)]);
            assert_relative_eq!(Euclidean.distance(&test_gc, &gc), 60.959002616512684);
        }
    } // End of original_distance_tests module

    // ┌─────────────────────────────────────────────────────────────────┐
    // │ Tests for DistanceExt trait (Generic WKB implementations)      │
    // └─────────────────────────────────────────────────────────────────┘

    mod distance_ext_tests {
        use super::*;

        #[test]
        fn distance_ext_point_to_point_test() {
            let p1 = Point::new(0., 0.);
            let p2 = Point::new(1., 0.);
            assert_relative_eq!(p1.distance_ext(&p2), 1.);
        }

        #[test]
        fn distance_ext_point_to_point_test_2() {
            let p1 = Point::new(-72.1235, 42.3521);
            let p2 = Point::new(72.1260, 70.612);
            let dist = p1.distance_ext(&p2);
            assert_relative_eq!(dist, 146.99163308930207);
        }

        #[test]
        fn distance_ext_point_to_point_distance_test() {
            // Test specific point distances that match original test cases
            let p1 = Point::new(2.5, 0.5);
            let p2 = Point::new(5., 1.);
            let dist = p1.distance_ext(&p2);
            // This should give us the distance between these two specific points
            assert!(dist > 0.0);
        }

        #[test]
        fn distance_ext_linestring_distance_test() {
            // Test LineString to LineString distances
            let points1 = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
            ];
            let points2 = vec![(8., 1.), (9., 2.), (9., 3.), (8., 4.)];
            let ls1 = LineString::from(points1);
            let ls2 = LineString::from(points2);
            let dist = ls1.distance_ext(&ls2);
            assert_relative_eq!(dist, std::f64::consts::SQRT_2);
        }

        #[test]
        fn distance_ext_linestring_contains_test() {
            // Test LineString to same LineString (should be 0)
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
            ];
            let ls = LineString::from(points);
            let dist = ls.distance_ext(&ls);
            assert_relative_eq!(dist, 0.0);
        }

        #[test]
        fn distance_ext_linestring_to_linestring_test() {
            let ls1: LineString<f64> = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)].into();
            let ls2: LineString<f64> = vec![(3.0, 0.0), (4.0, 1.0), (5.0, 0.0)].into();
            let dist = ls1.distance_ext(&ls2);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_polygon_to_polygon_test() {
            let points1 = vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)];
            let points2 = vec![(3., 0.), (5., 0.), (5., 2.), (3., 2.), (3., 0.)];
            let poly1 = Polygon::new(LineString::from(points1), vec![]);
            let poly2 = Polygon::new(LineString::from(points2), vec![]);
            let dist = poly1.distance_ext(&poly2);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_multipoint_test() {
            let v = vec![
                Point::new(0.0, 10.0),
                Point::new(1.0, 1.0),
                Point::new(10.0, 0.0),
                Point::new(1.0, -1.0),
                Point::new(0.0, -10.0),
                Point::new(-1.0, -1.0),
                Point::new(-10.0, 0.0),
                Point::new(-1.0, 1.0),
                Point::new(0.0, 10.0),
            ];
            let mp1 = MultiPoint::new(v.clone());
            let mp2 = MultiPoint::new(vec![Point::new(50.0, 50.0)]);
            let dist = mp1.distance_ext(&mp2);
            assert_relative_eq!(dist, 64.03124237432849);
        }

        #[test]
        fn distance_ext_multilinestring_test() {
            let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
            let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
            let mls1 = MultiLineString::new(vec![v1, v2]);

            let v3 = LineString::from(vec![(50.0, 50.0), (51.0, 60.0)]);
            let mls2 = MultiLineString::new(vec![v3]);

            let dist = mls1.distance_ext(&mls2);
            assert_relative_eq!(dist, 63.25345840347388);
        }

        #[test]
        fn distance_ext_multipolygon_test() {
            let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
            let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
            let p1 = Polygon::new(ls1, vec![]);
            let p2 = Polygon::new(ls2, vec![]);
            let mp1 = MultiPolygon::new(vec![p1, p2]);

            let ls3 =
                LineString::from(vec![(50.0, 50.0), (51.0, 60.0), (52.0, 50.0), (50.0, 50.0)]);
            let p3 = Polygon::new(ls3, vec![]);
            let mp2 = MultiPolygon::new(vec![p3]);

            let dist = mp1.distance_ext(&mp2);
            assert_relative_eq!(dist, 60.959002616512684);
        }

        #[test]
        fn distance_ext_triangle_test() {
            use geo_types::Triangle;
            let tri1 = Triangle::from([(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)]);
            let tri2 = Triangle::from([(3.0, 0.0), (5.0, 0.0), (4.0, 2.0)]);
            let dist = tri1.distance_ext(&tri2);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_rect_test() {
            use geo_types::Rect;
            let rect1 = Rect::new((0.0, 0.0), (2.0, 2.0));
            let rect2 = Rect::new((3.0, 0.0), (5.0, 2.0));
            let dist = rect1.distance_ext(&rect2);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_empty_geometry_test() {
            let empty_ls1: LineString<f64> = LineString::new(vec![]);
            let empty_ls2: LineString<f64> = LineString::new(vec![]);
            let dist = empty_ls1.distance_ext(&empty_ls2);
            assert_relative_eq!(dist, 0.0);
        }

        #[test]
        fn distance_ext_zero_distance_test() {
            // Test same point to itself
            let p = Point::new(1.0, 2.0);
            let dist = p.distance_ext(&p);
            assert_relative_eq!(dist, 0.0);

            // Test overlapping linestrings
            let ls = LineString::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
            let dist = ls.distance_ext(&ls);
            assert_relative_eq!(dist, 0.0);
        }

        #[test]
        fn distance_ext_symmetry_test() {
            // Test that distance is symmetric: dist(a, b) == dist(b, a)
            let p1 = Point::new(0.0, 0.0);
            let p2 = Point::new(3.0, 4.0);
            let dist1 = p1.distance_ext(&p2);
            let dist2 = p2.distance_ext(&p1);
            assert_relative_eq!(dist1, dist2);

            let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 1.0)]);
            let ls2 = LineString::from(vec![(2.0, 2.0), (3.0, 3.0)]);
            let dist3 = ls1.distance_ext(&ls2);
            let dist4 = ls2.distance_ext(&ls1);
            assert_relative_eq!(dist3, dist4);
        }

        // ┌─────────────────────────────────────────────────────────────────┐
        // │ Cross-type distance tests for DistanceExt                      │
        // └─────────────────────────────────────────────────────────────────┘

        #[test]
        fn distance_ext_point_to_linestring_test() {
            // Like an octagon, but missing the lowest horizontal segment
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
            ];
            let ls = LineString::from(points);
            // A Random point "inside" the LineString
            let p = Point::new(5.5, 2.1);
            let dist = distance_point_to_linestring_generic(&p, &ls);
            assert_relative_eq!(dist, 1.1313708498984762);
        }

        #[test]
        fn distance_ext_point_to_polygon_test() {
            // An octagon
            let points = vec![
                (5., 1.),
                (4., 2.),
                (4., 3.),
                (5., 4.),
                (6., 4.),
                (7., 3.),
                (7., 2.),
                (6., 1.),
                (5., 1.),
            ];
            let ls = LineString::from(points);
            let poly = Polygon::new(ls, vec![]);
            // A Random point outside the octagon
            let p = Point::new(2.5, 0.5);
            let dist = distance_point_to_polygon_generic(&p, &poly);
            assert_relative_eq!(dist, 2.1213203435596424);
        }

        #[test]
        fn distance_ext_linestring_to_point_test() {
            let ls = LineString::from(vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
            let p = Point::new(3.0, 1.0);
            let dist = distance_linestring_to_point_generic(&ls, &p);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_linestring_to_polygon_test() {
            let ls = LineString::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);
            let poly_points = vec![(3., 0.), (5., 0.), (5., 2.), (3., 2.), (3., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);
            let dist = distance_linestring_to_polygon_generic(&ls, &poly);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_polygon_to_point_test() {
            let poly_points = vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);
            let p = Point::new(3.0, 1.0);
            let dist = distance_polygon_to_point_generic(&poly, &p);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_polygon_to_linestring_test() {
            let poly_points = vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);
            let ls = LineString::from(vec![(3.0, 0.0), (4.0, 1.0), (5.0, 0.0)]);
            let dist = distance_polygon_to_linestring_generic(&poly, &ls);
            assert_relative_eq!(dist, 1.0);
        }

        #[test]
        fn distance_ext_cross_type_symmetry_test() {
            // Test that cross-type distance is symmetric via helper functions
            let p = Point::new(3.0, 4.0);
            let ls = LineString::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]);

            let dist1 = distance_point_to_linestring_generic(&p, &ls);
            let dist2 = distance_linestring_to_point_generic(&ls, &p);
            assert_relative_eq!(dist1, dist2);
        }

        #[test]
        fn distance_ext_rect_to_polygon_test() {
            // Test that Rect to Polygon cross-type distance is now supported via helper functions
            use geo_types::Rect;
            let rect = Rect::new((0.0, 0.0), (2.0, 2.0));
            let poly_points = vec![(3., 0.), (5., 0.), (5., 2.), (3., 2.), (3., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);

            // Test cross-type distance via conversion and helper functions
            let rect_poly = rect.to_polygon();
            let dist1 = distance_polygon_to_polygon_generic(&rect_poly, &poly);
            let dist2 = distance_polygon_to_polygon_generic(&poly, &rect_poly);
            assert_relative_eq!(dist1, 1.0);
            assert_relative_eq!(dist2, 1.0);
            assert_relative_eq!(dist1, dist2); // Verify symmetry
        }

        #[test]
        fn distance_ext_boundary_cases_test() {
            // Test point on polygon boundary
            let poly_points = vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)];
            let poly = Polygon::new(LineString::from(poly_points), vec![]);
            let p_on_boundary = Point::new(0.0, 1.0); // On left edge
            let dist = distance_point_to_polygon_generic(&p_on_boundary, &poly);
            assert_relative_eq!(dist, 0.0);
        }
    } // End of distance_ext_tests module
} // End of tests module
