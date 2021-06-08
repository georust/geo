use std::cmp::Ordering;

use crate::algorithm::area::{get_linestring_area, Area};
use crate::algorithm::dimensions::{Dimensions, Dimensions::*, HasDimensions};
use crate::algorithm::euclidean_length::EuclideanLength;
use crate::{
    Coordinate, GeoFloat, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// Calculation of the centroid.
/// The centroid is the arithmetic mean position of all points in the shape.
/// Informally, it is the point at which a cutout of the shape could be perfectly
/// balanced on the tip of a pin.
/// The geometric centroid of a convex object always lies in the object.
/// A non-convex object might have a centroid that _is outside the object itself_.
///
/// # Examples
///
/// ```
/// use geo::algorithm::centroid::Centroid;
/// use geo::{point, polygon};
///
/// // rhombus shaped polygon
/// let polygon = polygon![
///     (x: -2., y: 1.),
///     (x: 1., y: 3.),
///     (x: 4., y: 1.),
///     (x: 1., y: -1.),
///     (x: -2., y: 1.),
/// ];
///
/// assert_eq!(
///     Some(point!(x: 1., y: 1.)),
///     polygon.centroid(),
/// );
/// ```
pub trait Centroid {
    type Output;

    /// See: https://en.wikipedia.org/wiki/Centroid
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::centroid::Centroid;
    /// use geo::{line_string, point};
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 40.02f64, y: 118.23),
    /// ];
    ///
    /// assert_eq!(
    ///     Some(point!(x: 40.02, y: 117.285)),
    ///     line_string.centroid(),
    /// );
    /// ```
    fn centroid(&self) -> Self::Output;
}

impl<T> Centroid for Line<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        (self.start_point() + self.end_point()) / two
    }
}

impl<T> Centroid for LineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    // The Centroid of a LineString is the mean of the middle of the segment
    // weighted by the length of the segments.
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiLineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// The Centroid of a MultiLineString is the mean of the centroids of all the constituent linestrings,
    /// weighted by the length of each linestring
    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_line_string(self);
        operation.centroid()
    }
}

impl<T> Centroid for Polygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for MultiPolygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_polygon(self);
        operation.centroid()
    }
}

impl<T> Centroid for Rect<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        self.center().into()
    }
}

impl<T> Centroid for Point<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        *self
    }
}

///
/// ```
/// use geo::algorithm::centroid::Centroid;
/// use geo::{MultiPoint, Point};
///
/// let empty: Vec<Point<f64>> = Vec::new();
/// let empty_multi_points: MultiPoint<_> = empty.into();
/// assert_eq!(empty_multi_points.centroid(), None);
///
/// let points: MultiPoint<_> = vec![(5., 1.), (1., 3.), (3., 2.)].into();
/// assert_eq!(points.centroid(), Some(Point::new(3., 2.)));
/// ```
impl<T> Centroid for MultiPoint<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_multi_point(self);
        operation.centroid()
    }
}

impl<T> Centroid for Geometry<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    crate::geometry_delegate_impl! {
        fn centroid(&self) -> Self::Output;
    }
}

impl<T> Centroid for GeometryCollection<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_geometry_collection(self);
        operation.centroid()
    }
}

impl<T> Centroid for Triangle<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        let mut operation = CentroidOperation::new();
        operation.add_triangle(self);
        operation
            .centroid()
            .expect("triangle cannot have an empty centroid")
    }
}

struct CentroidOperation<T: GeoFloat>(Option<WeightedCentroid<T>>);
impl<T: GeoFloat> CentroidOperation<T> {
    fn new() -> Self {
        CentroidOperation(None)
    }

    fn centroid(&self) -> Option<Point<T>> {
        self.0.as_ref().map(|weighted_centroid| {
            Point(weighted_centroid.accumulated / weighted_centroid.weight)
        })
    }

    fn centroid_dimensions(&self) -> Dimensions {
        self.0
            .as_ref()
            .map(|weighted_centroid| weighted_centroid.dimensions)
            .unwrap_or(Empty)
    }

    fn add_coord(&mut self, coord: Coordinate<T>) {
        self.add_centroid(ZeroDimensional, coord, T::one());
    }

    fn add_line(&mut self, line: &Line<T>) {
        match line.dimensions() {
            ZeroDimensional => self.add_coord(line.start),
            OneDimensional => {
                self.add_centroid(OneDimensional, line.centroid().0, line.euclidean_length())
            }
            _ => unreachable!("Line must be zero or one dimensional"),
        }
    }

    fn add_line_string(&mut self, line_string: &LineString<T>) {
        if self.centroid_dimensions() > OneDimensional {
            return;
        }

        if line_string.0.len() == 1 {
            self.add_coord(line_string.0[0]);
            return;
        }

        for line in line_string.lines() {
            self.add_line(&line);
        }
    }

    fn add_multi_line_string(&mut self, multi_line_string: &MultiLineString<T>) {
        if self.centroid_dimensions() > OneDimensional {
            return;
        }

        for element in &multi_line_string.0 {
            self.add_line_string(element);
        }
    }

    fn add_polygon(&mut self, polygon: &Polygon<T>) {
        // Polygons which are completely covered by their interior rings have zero area, and
        // represent a unique degeneracy into a line_string which cannot be handled by accumulating
        // directly into `self`. Instead, we perform a sub-operation, inspect the result, and only
        // then incorporate the result into `self.

        let mut exterior_operation = CentroidOperation::new();
        exterior_operation.add_ring(polygon.exterior());

        let mut interior_operation = CentroidOperation::new();
        for interior in polygon.interiors() {
            interior_operation.add_ring(interior);
        }

        if let Some(exterior_weighted_centroid) = exterior_operation.0 {
            let mut poly_weighted_centroid = exterior_weighted_centroid;
            if let Some(interior_weighted_centroid) = interior_operation.0 {
                poly_weighted_centroid.sub_assign(interior_weighted_centroid);
                if poly_weighted_centroid.weight.is_zero() {
                    // A polygon with no area `interiors` completely covers `exterior`, degenerating to a linestring
                    self.add_line_string(polygon.exterior());
                    return;
                }
            }
            self.add_weighted_centroid(poly_weighted_centroid);
        }
    }

    fn add_multi_point(&mut self, multi_point: &MultiPoint<T>) {
        if self.centroid_dimensions() > ZeroDimensional {
            return;
        }

        for element in &multi_point.0 {
            self.add_coord(element.0);
        }
    }

    fn add_multi_polygon(&mut self, multi_polygon: &MultiPolygon<T>) {
        for element in &multi_polygon.0 {
            self.add_polygon(element);
        }
    }

    fn add_geometry_collection(&mut self, geometry_collection: &GeometryCollection<T>) {
        for element in &geometry_collection.0 {
            self.add_geometry(element);
        }
    }

    fn add_rect(&mut self, rect: &Rect<T>) {
        match rect.dimensions() {
            ZeroDimensional => self.add_coord(rect.min()),
            OneDimensional => {
                // Degenerate rect is a line, treat it the same way we treat flat polygons
                self.add_line(&Line::new(rect.min(), rect.min()));
                self.add_line(&Line::new(rect.min(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.max()));
                self.add_line(&Line::new(rect.max(), rect.min()));
            }
            TwoDimensional => {
                self.add_centroid(TwoDimensional, rect.centroid().0, rect.unsigned_area())
            }
            Empty => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_triangle(&mut self, triangle: &Triangle<T>) {
        match triangle.dimensions() {
            ZeroDimensional => self.add_coord(triangle.0),
            OneDimensional => {
                // Degenerate triangle is a line, treat it the same way we treat flat
                // polygons
                let l0_1 = Line::new(triangle.0, triangle.1);
                let l1_2 = Line::new(triangle.1, triangle.2);
                let l2_0 = Line::new(triangle.2, triangle.0);
                self.add_line(&l0_1);
                self.add_line(&l1_2);
                self.add_line(&l2_0);
            }
            TwoDimensional => {
                let centroid = (triangle.0 + triangle.1 + triangle.2) / T::from(3).unwrap();
                self.add_centroid(TwoDimensional, centroid, triangle.unsigned_area());
            }
            Empty => unreachable!("Rect dimensions cannot be empty"),
        }
    }

    fn add_geometry(&mut self, geometry: &Geometry<T>) {
        match geometry {
            Geometry::Point(g) => self.add_coord(g.0),
            Geometry::Line(g) => self.add_line(g),
            Geometry::LineString(g) => self.add_line_string(g),
            Geometry::Polygon(g) => self.add_polygon(g),
            Geometry::MultiPoint(g) => self.add_multi_point(g),
            Geometry::MultiLineString(g) => self.add_multi_line_string(g),
            Geometry::MultiPolygon(g) => self.add_multi_polygon(g),
            Geometry::GeometryCollection(g) => self.add_geometry_collection(g),
            Geometry::Rect(g) => self.add_rect(g),
            Geometry::Triangle(g) => self.add_triangle(g),
        }
    }

    fn add_ring(&mut self, ring: &LineString<T>) {
        debug_assert!(ring.is_closed());

        let area = get_linestring_area(ring);
        if area == T::zero() {
            match ring.dimensions() {
                // empty ring doesn't contribute to centroid
                Empty => {}
                // degenerate ring is a point
                ZeroDimensional => self.add_coord(ring[0]),
                // zero-area ring is a line string
                _ => self.add_line_string(ring),
            }
            return;
        }

        // Since area is non-zero, we know the ring has at least one point
        let shift = ring.0[0];

        let accumulated_coord = ring.lines().fold(Coordinate::zero(), |accum, line| {
            use crate::algorithm::map_coords::MapCoords;
            let line = line.map_coords(|&(x, y)| (x - shift.x, y - shift.y));
            let tmp = line.determinant();
            accum + (line.end + line.start) * tmp
        });
        let six = T::from(6).unwrap();
        let centroid = accumulated_coord / (six * area) + shift;
        let weight = area.abs();
        self.add_centroid(TwoDimensional, centroid, weight);
    }

    fn add_centroid(&mut self, dimensions: Dimensions, centroid: Coordinate<T>, weight: T) {
        let weighted_centroid = WeightedCentroid {
            dimensions,
            weight,
            accumulated: centroid * weight,
        };
        self.add_weighted_centroid(weighted_centroid);
    }

    fn add_weighted_centroid(&mut self, other: WeightedCentroid<T>) {
        match self.0.as_mut() {
            Some(centroid) => centroid.add_assign(other),
            None => self.0 = Some(other),
        }
    }
}

// Aggregated state for accumulating the centroid of a geometry or collection of geometries.
struct WeightedCentroid<T: GeoFloat> {
    weight: T,
    accumulated: Coordinate<T>,
    /// Collections of Geometries can have different dimensionality. Centroids must be considered
    /// separately by dimensionality.
    ///
    /// e.g. If I have several Points, adding a new `Point` will affect their centroid.
    ///
    /// However, because a Point is zero dimensional, it is infinitely small when compared to
    /// any 2-D Polygon. Thus a Point will not affect the centroid of any GeometryCollection
    /// containing a 2-D Polygon.
    ///
    /// So, when accumulating a centroid, we must track the dimensionality of the centroid
    dimensions: Dimensions,
}

impl<T: GeoFloat> WeightedCentroid<T> {
    fn add_assign(&mut self, b: WeightedCentroid<T>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated + b.accumulated;
                self.weight = self.weight + b.weight;
            }
        }
    }

    fn sub_assign(&mut self, b: WeightedCentroid<T>) {
        match self.dimensions.cmp(&b.dimensions) {
            Ordering::Less => *self = b,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.accumulated = self.accumulated - b.accumulated;
                self.weight = self.weight - b.weight;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, point, polygon};

    /// small helper to create a coordinate
    fn c<T: GeoFloat>(x: T, y: T) -> Coordinate<T> {
        Coordinate { x, y }
    }

    /// small helper to create a point
    fn p<T: GeoFloat>(x: T, y: T) -> Point<T> {
        Point(c(x, y))
    }

    // Tests: Centroid of LineString
    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = line_string![];
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let coord = Coordinate {
            x: 40.02f64,
            y: 116.34,
        };
        let linestring = line_string![coord];
        let centroid = linestring.centroid();
        assert_eq!(centroid, Some(Point(coord)));
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_eq!(linestring.centroid(), Some(point!(x: 6., y: 1. )));
    }
    #[test]
    fn linestring_with_repeated_point_test() {
        let l1 = LineString::from(vec![p(1., 1.), p(1., 1.), p(1., 1.)]);
        assert_eq!(l1.centroid(), Some(p(1., 1.)));

        let l2 = LineString::from(vec![p(2., 2.), p(2., 2.), p(2., 2.)]);
        let mls = MultiLineString(vec![l1, l2]);
        assert_eq!(mls.centroid(), Some(p(1.5, 1.5)));
    }
    // Tests: Centroid of MultiLineString
    #[test]
    fn empty_multilinestring_test() {
        let mls: MultiLineString<f64> = MultiLineString(vec![]);
        let centroid = mls.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn multilinestring_with_empty_line_test() {
        let mls: MultiLineString<f64> = MultiLineString(vec![line_string![]]);
        let centroid = mls.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn multilinestring_length_0_test() {
        let coord = Coordinate {
            x: 40.02f64,
            y: 116.34,
        };
        let mls: MultiLineString<f64> = MultiLineString(vec![
            line_string![coord],
            line_string![coord],
            line_string![coord],
        ]);
        assert_relative_eq!(mls.centroid().unwrap(), Point(coord));
    }
    #[test]
    fn multilinestring_one_line_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        let mls: MultiLineString<f64> = MultiLineString(vec![linestring]);
        assert_relative_eq!(mls.centroid().unwrap(), Point(Coordinate { x: 6., y: 1. }));
    }
    #[test]
    fn multilinestring_test() {
        let v1 = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0)];
        let v2 = line_string![(x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 3.0, y: 1.0)];
        let v3 = line_string![(x: -12.0, y: -100.0), (x: 7.0, y: 8.0)];
        let mls = MultiLineString(vec![v1, v2, v3]);
        assert_relative_eq!(
            mls.centroid().unwrap(),
            point![x: -1.9097834383655845, y: -37.683866439745714]
        );
    }
    // Tests: Centroid of Polygon
    #[test]
    fn empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert!(poly.centroid().is_none());
    }
    #[test]
    fn polygon_one_point_test() {
        let p = point![ x: 2., y: 1. ];
        let v = Vec::new();
        let linestring = line_string![p.0];
        let poly = Polygon::new(linestring, v);
        assert_relative_eq!(poly.centroid().unwrap(), p);
    }

    #[test]
    fn centroid_polygon_numerical_stability() {
        let polygon = {
            use std::f64::consts::PI;
            const NUM_VERTICES: usize = 10;
            const ANGLE_INC: f64 = 2. * PI / NUM_VERTICES as f64;

            Polygon::new(
                (0..NUM_VERTICES)
                    .map(|i| {
                        let angle = i as f64 * ANGLE_INC;
                        Coordinate {
                            x: angle.cos(),
                            y: angle.sin(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .into(),
                vec![],
            )
        };

        let centroid = polygon.centroid().unwrap();

        let shift_x = 1.5e8;
        let shift_y = 1.5e8;

        use crate::map_coords::MapCoords;
        let polygon = polygon.map_coords(|&(x, y)| (x + shift_x, y + shift_y));

        let new_centroid = polygon
            .centroid()
            .unwrap()
            .map_coords(|&(x, y)| (x - shift_x, y - shift_y));
        debug!("centroid {:?}", centroid.0);
        debug!("new_centroid {:?}", new_centroid.0);
        assert_relative_eq!(centroid.0.x, new_centroid.0.x, max_relative = 0.0001);
        assert_relative_eq!(centroid.0.y, new_centroid.0.y, max_relative = 0.0001);
    }

    #[test]
    fn polygon_test() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 2., y: 0.),
            (x: 2., y: 2.),
            (x: 0., y: 2.),
            (x: 0., y: 0.)
        ];
        assert_relative_eq!(poly.centroid().unwrap(), point![x:1., y:1.]);
    }
    #[test]
    fn polygon_hole_test() {
        // hexagon
        let ls1 = LineString::from(vec![
            (5.0, 1.0),
            (4.0, 2.0),
            (4.0, 3.0),
            (5.0, 4.0),
            (6.0, 4.0),
            (7.0, 3.0),
            (7.0, 2.0),
            (6.0, 1.0),
            (5.0, 1.0),
        ]);

        let ls2 = LineString::from(vec![(5.0, 1.3), (5.5, 2.0), (6.0, 1.3), (5.0, 1.3)]);

        let ls3 = LineString::from(vec![(5., 2.3), (5.5, 3.0), (6., 2.3), (5., 2.3)]);

        let p1 = Polygon::new(ls1, vec![ls2, ls3]);
        let centroid = p1.centroid().unwrap();
        assert_relative_eq!(centroid, point!(x: 5.5, y: 2.5518518518518523));
    }
    #[test]
    fn flat_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 1.), p(1., 1.), p(0., 1.)]),
            vec![],
        );
        assert_eq!(poly.centroid(), Some(p(0.5, 1.)));
    }
    #[test]
    fn multi_poly_with_flat_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(1., 0.), p(0., 0.)]),
            vec![],
        );
        let multipoly = MultiPolygon(vec![poly]);
        assert_eq!(multipoly.centroid(), Some(p(0.5, 0.)));
    }
    #[test]
    fn multi_poly_with_multiple_flat_polygon_test() {
        let p1 = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 3.), p(1., 1.)]),
            vec![],
        );
        let p2 = Polygon::new(
            LineString::from(vec![p(2., 2.), p(6., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon(vec![p1, p2]);
        assert_eq!(multipoly.centroid(), Some(p(3., 2.)));
    }
    #[test]
    fn multi_poly_with_only_points_test() {
        let p1 = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 1.), p(1., 1.)]),
            vec![],
        );
        assert_eq!(p1.centroid(), Some(p(1., 1.)));
        let p2 = Polygon::new(
            LineString::from(vec![p(2., 2.), p(2., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon(vec![p1, p2]);
        assert_eq!(multipoly.centroid(), Some(p(1.5, 1.5)));
    }
    #[test]
    fn multi_poly_with_one_ring_and_one_real_poly() {
        // if the multipolygon is composed of a 'normal' polygon (with an area not null)
        // and a ring (a polygon with a null area)
        // the centroid of the multipolygon is the centroid of the 'normal' polygon
        let normal = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 3.), p(3., 1.), p(1., 1.)]),
            vec![],
        );
        let flat = Polygon::new(
            LineString::from(vec![p(2., 2.), p(6., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon(vec![normal.clone(), flat]);
        assert_eq!(multipoly.centroid(), normal.centroid());
    }
    #[test]
    fn polygon_flat_interior_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString::from(vec![p(0., 0.), p(0., 1.), p(0., 0.)])],
        );
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn empty_interior_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString(vec![])],
        );
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn polygon_ring_test() {
        let square = LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]);
        let poly = Polygon::new(square.clone(), vec![square]);
        assert_eq!(poly.centroid(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn polygon_cell_test() {
        // test the centroid of polygon with a null area
        // this one a polygon with 2 interior polygon that makes a partition of the exterior
        let square = LineString::from(vec![p(0., 0.), p(0., 2.), p(2., 2.), p(2., 0.), p(0., 0.)]);
        let bottom = LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 1.), p(0., 1.), p(0., 0.)]);
        let top = LineString::from(vec![p(0., 1.), p(2., 1.), p(2., 2.), p(0., 2.), p(0., 1.)]);
        let poly = Polygon::new(square, vec![top, bottom]);
        assert_eq!(poly.centroid(), Some(p(1., 1.)));
    }
    // Tests: Centroid of MultiPolygon
    #[test]
    fn empty_multipolygon_polygon_test() {
        assert!(MultiPolygon::<f64>(Vec::new()).centroid().is_none());
    }
    #[test]
    fn multipolygon_one_polygon_test() {
        let linestring =
            LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(MultiPolygon(vec![poly]).centroid(), Some(p(1., 1.)));
    }
    #[test]
    fn multipolygon_two_polygons_test() {
        let linestring =
            LineString::from(vec![p(2., 1.), p(5., 1.), p(5., 3.), p(2., 3.), p(2., 1.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring =
            LineString::from(vec![p(7., 1.), p(8., 1.), p(8., 2.), p(7., 2.), p(7., 1.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        let centroid = MultiPolygon(vec![poly1, poly2]).centroid().unwrap();
        assert_relative_eq!(
            centroid,
            point![x: 4.071428571428571, y: 1.9285714285714286]
        );
    }
    #[test]
    fn multipolygon_two_polygons_of_opposite_clockwise_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString::from(vec![(0., 0.), (-2., 0.), (-2., 2.), (0., 2.), (0., 0.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        assert_relative_eq!(
            MultiPolygon(vec![poly1, poly2]).centroid().unwrap(),
            point![x: 0., y: 1.]
        );
    }
    #[test]
    fn bounding_rect_test() {
        let bounding_rect = Rect::new(Coordinate { x: 0., y: 50. }, Coordinate { x: 4., y: 100. });
        let point = point![x: 2., y: 75.];
        assert_eq!(point, bounding_rect.centroid());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(c(0., 1.), c(1., 3.));
        assert_eq!(line1.centroid(), point![x: 0.5, y: 2.]);
    }
    #[test]
    fn collection_weighting() {
        let p0 = point!(x: 0.0, y: 0.0);
        let p1 = point!(x: 2.0, y: 0.0);
        let p2 = point!(x: 2.0, y: 2.0);
        let p3 = point!(x: 0.0, y: 2.0);

        let multi_point = MultiPoint(vec![p0, p1, p2, p3]);
        assert_eq!(multi_point.centroid().unwrap(), point!(x: 1.0, y: 1.0));

        let collection = GeometryCollection(vec![MultiPoint(vec![p1, p2, p3]).into(), p0.into()]);

        assert_eq!(collection.centroid().unwrap(), point!(x: 1.0, y: 1.0));
    }
    #[test]
    fn triangles() {
        // boring triangle
        assert_eq!(
            Triangle(c(0., 0.), c(3., 0.), c(1.5, 3.)).centroid(),
            point!(x: 1.5, y: 1.0)
        );

        // flat triangle
        assert_eq!(
            Triangle(c(0., 0.), c(3., 0.), c(1., 0.)).centroid(),
            point!(x: 1.5, y: 0.0)
        );

        // flat triangle that's not axis-aligned
        assert_eq!(
            Triangle(c(0., 0.), c(3., 3.), c(1., 1.)).centroid(),
            point!(x: 1.5, y: 1.5)
        );

        // triangle with some repeated points
        assert_eq!(
            Triangle(c(0., 0.), c(0., 0.), c(1., 0.)).centroid(),
            point!(x: 0.5, y: 0.0)
        );

        // triangle with all repeated points
        assert_eq!(
            Triangle(c(0., 0.5), c(0., 0.5), c(0., 0.5)).centroid(),
            point!(x: 0., y: 0.5)
        )
    }

    #[test]
    fn degenerate_triangle_like_ring() {
        let triangle = Triangle(c(0., 0.), c(1., 1.), c(2., 2.));
        let poly: Polygon<_> = triangle.clone().into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection(vec![triangle.into(), line.into()]);
        let g2 = GeometryCollection(vec![poly.into(), line.into()]);
        assert_eq!(g1.centroid(), g2.centroid());
    }

    #[test]
    fn degenerate_rect_like_ring() {
        let rect = Rect::new(c(0., 0.), c(0., 4.));
        let poly: Polygon<_> = rect.clone().into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection(vec![rect.into(), line.into()]);
        let g2 = GeometryCollection(vec![poly.into(), line.into()]);
        assert_eq!(g1.centroid(), g2.centroid());
    }

    #[test]
    fn rectangles() {
        // boring rect
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 4.)).centroid(),
            point!(x: 2.0, y: 2.0)
        );

        // flat rect
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 0.)).centroid(),
            point!(x: 2.0, y: 0.0)
        );

        // rect with all repeated points
        assert_eq!(
            Rect::new(c(4., 4.), c(4., 4.)).centroid(),
            point!(x: 4., y: 4.)
        );

        // collection with rect
        let mut collection =
            GeometryCollection(vec![p(0., 0.).into(), p(6., 0.).into(), p(6., 6.).into()]);
        // sanity check
        assert_eq!(collection.centroid().unwrap(), point!(x: 4., y: 2.));

        // 0-d rect treated like point
        collection.0.push(Rect::new(c(0., 6.), c(0., 6.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 3., y: 3.));

        // 1-d rect treated like line. Since a line has higher dimensions than the rest of the
        // collection, it's centroid clobbers everything else in the collection.
        collection.0.push(Rect::new(c(0., 0.), c(0., 2.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 0., y: 1.));

        // 2-d has higher dimensions than the rest of the collection, so it's centroid clobbers
        // everything else in the collection.
        collection
            .0
            .push(Rect::new(c(10., 10.), c(11., 11.)).into());
        assert_eq!(collection.centroid().unwrap(), point!(x: 10.5, y: 10.5));
    }
}
