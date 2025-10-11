use super::{Distance, Euclidean};
use crate::Centroid;
use crate::HasDimensions;
use crate::algorithm::BoundingRect;
use crate::algorithm::Intersects;
use crate::coordinate_position::{CoordPos, coord_pos_relative_to_ring};
use crate::geometry::*;
use crate::{CoordFloat, GeoFloat, GeoNum};
use num_traits::{Bounded, Float};
use rstar::RTree;
use rstar::primitives::CachedEnvelope;
use std::ops::ControlFlow;

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
    /// - `origin`, `destination`: Point where the units of x/y represent non-angular units,
    ///   e.g. meters or miles, not lon/lat. For lon/lat points, use the
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

        // REVIEW: This impl changed slightly.
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
            return F::zero();
        }
        if line_string_a.0.is_empty() || line_string_b.0.is_empty() {
            return F::zero();
        }

        // Check if bounding boxes are non-overlapping: if so, use project-and-prune optimisation
        // Safety: both linestrings have been checked for emptiness, so bounding_rect() will return Some
        let rect_a = line_string_a.bounding_rect().unwrap();
        let rect_b = line_string_b.bounding_rect().unwrap();

        // Check for bbox separation along both axes
        let x_separated = rect_a.max().x < rect_b.min().x || rect_b.max().x < rect_a.min().x;
        let y_separated = rect_a.max().y < rect_b.min().y || rect_b.max().y < rect_a.min().y;

        // Ensure geometries meet minimum size requirements for the fast algorithm
        // LineStrings need at least 2 coordinates to form valid segments
        let has_min_coords = line_string_a.0.len() >= 2 && line_string_b.0.len() >= 2;

        if (x_separated || y_separated) && has_min_coords {
            return separable_geometry_distance_fast(line_string_a, line_string_b, rect_a, rect_b);
        }

        nearest_neighbour_distance(line_string_a, line_string_b)
    }
}

impl<F: GeoFloat> Distance<F, &LineString<F>, &Polygon<F>> for Euclidean {
    fn distance(&self, line_string: &LineString<F>, polygon: &Polygon<F>) -> F {
        if line_string.intersects(polygon) {
            return F::zero();
        }
        if line_string.0.is_empty() || polygon.exterior().0.is_empty() {
            return F::zero();
        }

        // Check if bounding boxes are non-overlapping: if so, use project-and-prune optimisation
        // Safety: both geometries have been checked for emptiness, so bounding_rect() will return Some
        let rect_ls = line_string.bounding_rect().unwrap();
        let rect_poly = polygon.bounding_rect().unwrap();

        // Check for bbox separation along both axes
        let x_separated =
            rect_ls.max().x < rect_poly.min().x || rect_poly.max().x < rect_ls.min().x;
        let y_separated =
            rect_ls.max().y < rect_poly.min().y || rect_poly.max().y < rect_ls.min().y;

        // Ensure geometries meet minimum size requirements for the fast algorithm
        // LineStrings need at least 2 coordinates; Polygons need at least 4 (triangle + closing vertex)
        let has_min_coords = line_string.0.len() >= 2 && polygon.exterior().0.len() >= 4;

        if (x_separated || y_separated) && has_min_coords {
            return separable_geometry_distance_fast(
                line_string,
                polygon.exterior(),
                rect_ls,
                rect_poly,
            );
        }

        if !polygon.interiors().is_empty()
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
        if polygon_a.is_empty() || polygon_b.is_empty() {
            return F::zero();
        }

        // Check if bounding boxes are non-overlapping: if so, use project-and-sort optimisation
        // Safety: both polygons have been checked for emptiness, so bounding_rect() will return Some
        let rect_a = polygon_a.bounding_rect().unwrap();
        let rect_b = polygon_b.bounding_rect().unwrap();

        // Check for bbox separation along both axes
        // TODO: do we have anything built-in that does this cheaply?
        let x_separated = rect_a.max().x < rect_b.min().x || rect_b.max().x < rect_a.min().x;
        let y_separated = rect_a.max().y < rect_b.min().y || rect_b.max().y < rect_a.min().y;

        // Ensure geometries meet minimum size requirements for the fast algorithm
        // Polygons need at least 4 coordinates (triangle + closing vertex)
        let has_min_coords = polygon_a.exterior().0.len() >= 4 && polygon_b.exterior().0.len() >= 4;

        if (x_separated || y_separated) && has_min_coords {
            return separable_geometry_distance_fast(
                polygon_a.exterior(),
                polygon_b.exterior(),
                rect_a,
                rect_b,
            );
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

/// A geometry vertex with its 1D projection value
///
/// This structure maintains the mapping between a vertex's position in the
/// original geometry and its 1D projection value
#[derive(Clone, Copy, Debug)]
struct ProjectedVertex<F: GeoFloat> {
    /// The 1D projection value of this vertex
    intercept: F,
    /// Index into original geometry
    vertex_idx: usize,
}

/// Optimized minimum distance calculation for linearly-separable geometries
///
/// # Algorithm Overview
///
/// Let the geometries be named `P` and `Q`
///
/// 1. **Slope Calculation and Projection Axis Selection**: Calculate the vector between `P` and `Q` bbox
///    centroids and determine the slope of lines perpendicular to this connector. This slope is constant.
///
/// 2. **Vertex Projection**: Project all vertices from `P` and `Q` into 1D space by calculating where a line
///    through each vertex (with the perpendicular slope) intercepts either the `x` axis or `y` axis
///
/// 3. **Sorting**: Sort `P` and `Q`'s intercept values, maintaining
///    an index to their associated vertex coordinates
///
/// 4. **Pruned Search**: Iterate through all `PQ` vertex pairs in sorted order, using early
///    termination to skip full distance calculation for pairs whose intercept difference exceeds
///    the current minimum distance, updating it when a new minimum is found.
///    P is iterated from last to first (i.e. in reverse) and Q is iterated forwards.
///    If we encounter a gap larger than `max_projection_delta`, we can safely break from the inner
///    loop and move to the next vertex on P
///
/// # Projection Mathematics
///
/// The algorithm uses "lines" perpendicular to the connector between bbox centroids.
/// If the centroid-to-centroid vector is `(dx, dy)`, the perpendicular slope is:
/// - If `|dx| < |dy|` (connector more vertical): `slope = -dx/dy` (perpendicular is horizontal)
/// - If `|dx| ≥ |dy|` (connector more horizontal): `slope = -dy/dx` (perpendicular is vertical)
///
/// Each "line" is drawn by plotting a line (with the perpendicular slope) through a vertex
/// and its axis intercept (either `x` or `y`, see above).
///
/// # Early Termination
///
/// The algorithm maintains a `max_projection_delta` threshold calculated as:
/// `min_distance * sqrt(1 + slope²)` (see step 4b, below)
///
/// This factor enables pruning by relating `PQ` vertex pair intercept differences to minimum
/// possible distances.
///
/// ## Why the Factor Works
///
/// Each vertex lies on a line with slope `k` (perpendicular to the connector). These parallel
/// lines are distinguished by their axis intercepts: this is what we calculate and sort by.
///
/// If two `PQ` pair vertices have intercept difference Δi, their minimum possible Euclidean distance
/// occurs when the minimum distance between the vertices is perpendicular to these parallel lines:
/// - Moving 1 unit perpendicular to the lines changes actual distance by 1 unit
/// - But changes the intercept by `sqrt(1 + slope²)` units
///
/// Therefore: `perpendicular_distance = intercept_difference / sqrt(1 + slope²)`
///
/// ## Pruning
///
/// Given current minimum distance `d_min`:
/// - Any closer pair must have perpendicular separation `< d_min`
/// - Therefore their intercept difference must be `< d_min * sqrt(1 + slope²)`
///
/// If two `PQ` pair vertices have intercept difference `> max_projection_delta`, they CANNOT be closer
/// than `d_min` regardless of where they lie along their respective parallel lines. This is
/// conservative but guarantees we never skip a potentially optimal pair.
///
/// # Performance
///
/// - Time complexity: `O(n log n)`
///
/// # References
/// https://www.crunchydata.com/blog/inside-postgis-calculating-distance
fn separable_geometry_distance_fast<F: GeoFloat>(
    linestring_p: &LineString<F>,
    linestring_q: &LineString<F>,
    bbox_p: Rect<F>,
    bbox_q: Rect<F>,
) -> F {
    // Calculate bounding box centroids.
    let centroid_p = bbox_p.centroid();
    let centroid_q = bbox_q.centroid();

    let delta_x = centroid_q.x() - centroid_p.x();
    let delta_y = centroid_q.y() - centroid_p.y();

    // this is the slope (the `m` in `y = mx + b`) of lines that are perpendicular
    // to the bbox centroid connector.
    let (slope, use_x_projection) = if delta_x.abs() < delta_y.abs() {
        // Midpoint connection is more vertical → use horizontal-favouring projection
        (-delta_x / delta_y, false)
    } else {
        // Midpoint connection is more horizontal → use vertical-favouring projection
        (-delta_y / delta_x, true)
    };

    // Convenient access to the coordinate slices
    let p_coords = &linestring_p.0;
    let q_coords = &linestring_q.0;

    // Step 1: Project all vertices into 1D space
    // This gives us intercepts + index of original vertex
    let mut projected_vertices_p = calculate_vertex_intercepts(p_coords, slope, use_x_projection);
    let mut projected_vertices_q = calculate_vertex_intercepts(q_coords, slope, use_x_projection);

    // Step 2: Sort vertices by intercepts for spatial locality
    projected_vertices_p.sort_unstable_by(|a, b| a.intercept.total_cmp(&b.intercept));
    projected_vertices_q.sort_unstable_by(|a, b| a.intercept.total_cmp(&b.intercept));

    // Step 3: Determine which geometry is "left" (lower bbox centroid intercept value) vs "right"
    // (higher bbox centroid intercept value). This is critical for the iteration filter step to work efficiently
    let centroid_p_projection = if use_x_projection {
        centroid_p.x() - slope * centroid_p.y()
    } else {
        centroid_p.y() - slope * centroid_p.x()
    };
    let centroid_q_projection = if use_x_projection {
        centroid_q.x() - slope * centroid_q.y()
    } else {
        centroid_q.y() - slope * centroid_q.x()
    };
    // the geometry whose midpoint has the lower projection value becomes
    // the "left" geometry.
    let (left_intercepts, right_intercepts, left_coords, right_coords) =
        if centroid_p_projection < centroid_q_projection {
            (
                &projected_vertices_p,
                &projected_vertices_q,
                p_coords,
                q_coords,
            )
        } else {
            (
                &projected_vertices_q,
                &projected_vertices_p,
                q_coords,
                p_coords,
            )
        };

    // Step 4a: use the minimum distance between the segments containing
    // the first vertex pair we'll check as the initial lower distance
    // This corresponds to the highest intercept from left_list and lowest intercept from right_list
    //
    // NOTE: this was initially a point-point distance calculation
    // and the bound probably tightens within a few iterations
    // but we have the technology, so I'm erring on the side of less divergent logic
    let highest_left = left_intercepts
        .last()
        .expect("left intercepts should not be empty")
        .vertex_idx;
    let lowest_right = right_intercepts
        .first()
        .expect("right intercepts should not be empty")
        .vertex_idx;
    let min_distance =
        get_min_segment_distance(left_coords, highest_left, right_coords, lowest_right);

    // Step 4b: calculate the upper bound for a projection delta that could yield a smaller distance
    // This threshold allows us to skip vertex pairs that are too far apart by breaking early
    // this is the key piece of the algorithm!
    let max_projection_delta = min_distance * (F::one() + (slope * slope)).sqrt();

    // Step 5: minimum distance calculation. This is a bit hairy as an iterator, but the logic is straightforward:
    //
    // First: geometry vertex order: the vertices are ordered by their intercepts, NOT in original order!
    // We iterate through left geometry vertices in reverse (high→low intercept values)
    // and for each one, iterate forward through right geometry vertices (low→high).
    // 1. We start from the vertices that are closest together in 1D space
    // (high values from left meeting low values from right)
    // 2. As we iterate, the gap between projection values grows
    // 3. We break whenever the gap exceeds our threshold
    // 4. If we find a new minimum distance, store it and update the threshold.
    let result = left_intercepts.iter().rev().try_fold(
        (min_distance, max_projection_delta),
        |(min_dist, max_delta), vertex1| {
            // Outer loop early termination: skip if remaining vertices are too far away.
            // Uses strict inequality to check if the best-case scenario (smallest possible gap)
            // is already too large, in which case we can skip the entire inner loop.
            if right_intercepts[0].intercept - vertex1.intercept > max_delta {
                return ControlFlow::Break((min_dist, max_delta));
            }

            // Inner loop with early termination
            let inner_result = right_intercepts.iter().try_fold(
                (min_dist, max_delta),
                |(mut min_d, mut max_d), vertex2| {
                    // Inner loop early termination: skip vertices beyond threshold.
                    // Uses non-strict inequality (>=) as we iterate through increasingly distant points:
                    // Once we reach OR exceed the threshold, this point and all subsequent points are too far.
                    if vertex2.intercept - vertex1.intercept >= max_d {
                        return ControlFlow::Break((min_d, max_d));
                    }

                    // Calculate minimum distance between segments adjacent to these vertices
                    let dist = get_min_segment_distance(
                        left_coords,
                        vertex1.vertex_idx,
                        right_coords,
                        vertex2.vertex_idx,
                    );

                    if dist < min_d {
                        min_d = dist;
                        // Update threshold when we find a closer distance
                        max_d = (min_d * min_d * (F::one() + slope * slope)).sqrt();
                    }
                    ControlFlow::Continue((min_d, max_d))
                },
            );

            let (new_min, new_delta) = match inner_result {
                ControlFlow::Continue(values) | ControlFlow::Break(values) => values,
            };

            ControlFlow::Continue((new_min, new_delta))
        },
    );

    match result {
        ControlFlow::Continue((min, _)) | ControlFlow::Break((min, _)) => min,
    }
}

/// Projects vertices into 1D space (their intercept, given a slope and axis)
/// The slope is the perpendicular to the `PQ` bbox centroid connecting line. Either `x` or
/// `y` axis would work, but one is chosen to avoid division by zero errors / small values causing fp
/// issues.
///
/// # Notes
/// This function excludes the duplicate closing vertex in polygon rings to avoid
/// redundant calculations, as the first and last vertices are identical in closed polygons.
fn calculate_vertex_intercepts<F: GeoFloat>(
    coords: &[Coord<F>],
    perpendicular_slope: F,
    use_x_intercept: bool,
) -> Vec<ProjectedVertex<F>> {
    // Skip the last coordinate as it duplicates the first in closed polygon rings.
    // We maintain the original index for later segment construction.
    coords[..coords.len().saturating_sub(1)]
        .iter()
        .enumerate()
        .map(|(idx, &coord)| {
            // Calculate where a line through this vertex (with the perpendicular slope)
            // intercepts either the x-axis or y-axis.
            // This is the rearranged line equation: given y = mx + b, we solve for b.
            let intercept = if use_x_intercept {
                // For nearly vertical perpendiculars, find x-intercept
                // From x = my + b, we get b = x - my
                coord.x - perpendicular_slope * coord.y
            } else {
                // For nearly horizontal perpendiculars, find y-intercept
                // From y = mx + b, we get b = y - mx
                coord.y - perpendicular_slope * coord.x
            };
            ProjectedVertex {
                intercept,
                vertex_idx: idx,
            }
        })
        .collect()
}

/// Calculates the minimum Euclidean distance between segments adjacent to two vertices
///
/// For each vertex in a geometry, there are adjacent segments (edges) connecting
/// it to neighbouring vertices. This function finds all possible segment
/// combinations between two vertices and returns their minimum distance.
///
/// # Algorithm
///
/// For each vertex, we identify 2 adjacent segments:
/// - Closed ring (polygon): prev and next with wraparound
/// - Open linestring at endpoint: duplicate the single adjacent segment
/// - Open linestring at middle vertex: prev and next without wraparound
///
/// Then we compute distances between all 4 combinations.
///
/// # Edge Cases
///
/// - Closed rings: last coordinate duplicates first, vertices wrap around
/// - Open linestrings at endpoints: both array entries contain the same segment
#[inline(always)]
fn get_min_segment_distance<F: GeoFloat>(
    coords_p: &[Coord<F>],
    vertex_idx_p: usize,
    coords_q: &[Coord<F>],
    vertex_idx_q: usize,
) -> F {
    // Detect if geometry is closed (ring) or open (linestring)
    let is_closed_p = coords_p.first() == coords_p.last();
    let is_closed_q = coords_q.first() == coords_q.last();

    // Helper to compute adjacent segment indices for a vertex
    let get_segment_indices =
        |coords: &[Coord<F>], vertex_idx: usize, is_closed: bool| -> (usize, usize) {
            if is_closed {
                // Closed ring: wraparound logic
                let n = coords.len() - 1; // Exclude duplicate closing vertex
                let prev = if vertex_idx == 0 {
                    n - 1
                } else {
                    vertex_idx - 1
                };
                let next = if vertex_idx >= n - 1 {
                    0
                } else {
                    vertex_idx + 1
                };
                (prev, next)
            } else {
                // Open linestring: duplicate segment at endpoints
                let n = coords.len();
                if vertex_idx == 0 {
                    // First vertex: both prev and next use segment [0,1]
                    (0, 1)
                } else if vertex_idx == n - 1 {
                    // Last vertex: both prev and next use segment [n-2, n-1]
                    (n - 2, n - 1)
                } else {
                    // Middle vertex: normal prev/next
                    (vertex_idx - 1, vertex_idx + 1)
                }
            }
        };

    // Get segment indices for both geometries
    let (prev_idx_p, next_idx_p) = get_segment_indices(coords_p, vertex_idx_p, is_closed_p);
    let (prev_idx_q, next_idx_q) = get_segment_indices(coords_q, vertex_idx_q, is_closed_q);

    // Build segment arrays
    // separate arrays means it's easy to compute the cross product, and we won't have to alter
    // the iterator if we ever expand the number of segments
    let segments_p = [
        Line::new(coords_p[prev_idx_p], coords_p[vertex_idx_p]),
        Line::new(coords_p[vertex_idx_p], coords_p[next_idx_p]),
    ];
    let segments_q = [
        Line::new(coords_q[prev_idx_q], coords_q[vertex_idx_q]),
        Line::new(coords_q[vertex_idx_q], coords_q[next_idx_q]),
    ];

    // Find minimum distance between all segment combinations
    segments_p
        .iter()
        .flat_map(|seg_p| {
            segments_q
                .iter()
                .map(move |seg_q| Euclidean.distance(seg_p, seg_q))
        })
        .fold(Bounded::max_value(), |acc, dist| acc.min(dist))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::orient::{Direction, Orient};
    use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
    use geo_types::{coord, polygon, private_utils::line_segment_distance};

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
        let dist = Euclidean.distance(&Point::new(-72.1235, 42.3521), &Point::new(72.1260, 70.612));
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
}
