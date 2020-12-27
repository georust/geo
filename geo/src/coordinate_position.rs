use crate::algorithm::{
    bounding_rect::BoundingRect, dimensions::HasDimensions, intersects::Intersects,
    kernels::HasKernel,
};
use crate::{
    Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// The position of a `Coordinate` relative to a `Geometry`
#[derive(PartialEq, Clone, Debug)]
pub enum CoordPos {
    OnBoundary,
    Inside,
    Outside,
}

/// Determine whether a `Coordinate` lies inside, outside, or on the boundary of a geometry.
///
/// # Examples
///
/// ```rust
/// use geo::{polygon, Coordinate};
/// use geo::coordinate_position::{CoordinatePosition, CoordPos};
///
/// let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];
///
/// let inside_coord = Coordinate { x: 1.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&inside_coord), CoordPos::Inside);
///
/// let boundary_coord = Coordinate { x: 0.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&boundary_coord), CoordPos::OnBoundary);
///
/// let outside_coord = Coordinate { x: 5.0, y: 5.0 };
/// assert_eq!(square_poly.coordinate_position(&outside_coord), CoordPos::Outside);
/// ```
pub trait CoordinatePosition {
    type Scalar: HasKernel;
    fn coordinate_position(&self, coord: &Coordinate<Self::Scalar>) -> CoordPos {
        let mut is_inside = false;
        let mut boundary_count = 0;

        self.calculate_coordinate_position(coord, &mut is_inside, &mut boundary_count);

        // “The boundary of an arbitrary collection of geometries whose interiors are disjoint
        // consists of geometries drawn from the boundaries of the element geometries by
        // application of the ‘mod 2’ union rule”
        //
        // ― OpenGIS Simple Feature Access § 6.1.15.1
        if boundary_count % 2 == 1 {
            CoordPos::OnBoundary
        } else if is_inside {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        }
    }

    // impls of this trait must:
    //  1. set `is_inside = true` if `coord` is contained within the Interior of any component.
    //  2. increment `boundary_count` for each component whose Boundary contains `coord`.
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    );
}

impl<T> CoordinatePosition for Coordinate<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self == coord {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Point<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if &self.0 == coord {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Line<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        // degenerate line is a point
        if self.start == self.end {
            self.start
                .calculate_coordinate_position(coord, is_inside, boundary_count);
            return;
        }

        if coord == &self.start || coord == &self.end {
            *boundary_count += 1;
        } else if self.intersects(coord) {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for LineString<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.0.len() < 2 {
            debug_assert!(false, "invalid line string with less than 2 coords");
            return;
        }

        if self.0.len() == 2 {
            // line string with two coords is just a line
            Line::new(self.0[0], self.0[1]).calculate_coordinate_position(
                coord,
                is_inside,
                boundary_count,
            );
            return;
        }

        // optimization: return early if there's no chance of an intersection
        // since self.0 is non-empty, safe to `unwrap`
        if !self.bounding_rect().unwrap().intersects(coord) {
            return;
        }

        // A closed linestring has no boundary, per SFS
        if !self.is_closed() {
            // since self.0 is non-empty, safe to `unwrap`
            if coord == self.0.first().unwrap() || coord == self.0.last().unwrap() {
                *boundary_count += 1;
                return;
            }
        }

        if self.intersects(coord) {
            // We've already checked for "Boundary" condition, so if there's an intersection at
            // this point, coord must be on the interior
            *is_inside = true
        }
    }
}

impl<T> CoordinatePosition for Triangle<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        // PERF TODO: I'm sure there's a better way to calculate than converting to a polygon
        self.to_polygon()
            .calculate_coordinate_position(coord, is_inside, boundary_count);
    }
}

impl<T> CoordinatePosition for Rect<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        // PERF TODO: I'm sure there's a better way to calculate than converting to a polygon
        self.to_polygon()
            .calculate_coordinate_position(coord, is_inside, boundary_count);
    }
}

impl<T> CoordinatePosition for MultiPoint<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self.0.iter().any(|p| &p.0 == coord) {
            *is_inside = true;
        }
    }
}

impl<T> CoordinatePosition for Polygon<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.is_empty() {
            return;
        }

        // Ok to `unwrap` since we know `self` is non-empty, so bounding-rect is non-null
        if !self.bounding_rect().unwrap().intersects(coord) {
            return;
        }

        match coord_pos_relative_to_ring(*coord, self.exterior()) {
            CoordPos::Outside => {}
            CoordPos::OnBoundary => {
                *boundary_count += 1;
            }
            CoordPos::Inside => {
                for hole in self.interiors() {
                    match coord_pos_relative_to_ring(*coord, hole) {
                        CoordPos::Outside => {}
                        CoordPos::OnBoundary => {
                            *boundary_count += 1;
                            return;
                        }
                        CoordPos::Inside => {
                            return;
                        }
                    }
                }
                // the coord is *outside* the interior holes, so it's *inside* the polygon
                *is_inside = true;
            }
        }
    }
}

impl<T> CoordinatePosition for MultiLineString<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for line_string in &self.0 {
            line_string.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for MultiPolygon<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for polygon in &self.0 {
            polygon.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for GeometryCollection<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for geometry in self {
            geometry.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T> CoordinatePosition for Geometry<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.is_empty() {
            return;
        }

        match self {
            Geometry::Point(point) => {
                point.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::Line(line) => {
                line.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::LineString(line_string) => {
                line_string.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::Polygon(polygon) => {
                polygon.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::Rect(rect) => {
                rect.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::Triangle(triangle) => {
                triangle.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::MultiPoint(multi_point) => {
                multi_point.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::MultiLineString(multi_line_string) => {
                multi_line_string.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::MultiPolygon(multi_polygon) => {
                multi_polygon.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
            Geometry::GeometryCollection(geometry_collection) => {
                geometry_collection.calculate_coordinate_position(coord, is_inside, boundary_count)
            }
        }
    }
}

/// Calculate the position of a `Coordinate` relative to a
/// closed `LineString`.
pub fn coord_pos_relative_to_ring<T>(coord: Coordinate<T>, linestring: &LineString<T>) -> CoordPos
where
    T: HasKernel,
{
    // Use the ray-tracing algorithm: count #times a
    // horizontal ray from point (to positive infinity).
    //
    // See: https://en.wikipedia.org/wiki/Point_in_polygon

    debug_assert!(linestring.is_closed());

    // LineString without points
    if linestring.0.is_empty() {
        return CoordPos::Outside;
    }
    if linestring.0.len() == 1 {
        // If LineString has one point, it will not generate
        // any lines.  So, we handle this edge case separately.
        return if coord == linestring.0[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    let mut crossings = 0;
    for line in linestring.lines() {
        // Check if coord lies on the line
        if line.intersects(&coord) {
            return CoordPos::OnBoundary;
        }

        // Ignore if the line is strictly to the left of the coord.
        let max_x = if line.start.x < line.end.x {
            line.end.x
        } else {
            line.start.x
        };
        if max_x < coord.x {
            continue;
        }

        // Ignore if line is horizontal. This includes an
        // edge case where the ray would intersect a
        // horizontal segment of the ring infinitely many
        // times, and is irrelevant for the calculation.
        if line.start.y == line.end.y {
            continue;
        }

        // Ignore if the intersection of the line is
        // possibly at the beginning/end of the line, and
        // the line lies below the ray. This is to
        // prevent a double counting when the ray passes
        // through a vertex of the polygon.
        //
        // The below logic handles two cases:
        //   1. if the ray enters/exits the polygon
        //      at the point of intersection
        //   2. if the ray touches a vertex,
        //      but doesn't enter/exit at that point
        if (line.start.y == coord.y && line.end.y < coord.y)
            || (line.end.y == coord.y && line.start.y < coord.y)
        {
            continue;
        }

        // Otherwise, check if ray intersects the line
        // segment. Enough to consider ray upto the max_x
        // coordinate of the current segment.
        let ray = Line::new(
            coord,
            Coordinate {
                x: max_x,
                y: coord.y,
            },
        );
        if ray.intersects(&line) {
            crossings += 1;
        }
    }
    if crossings % 2 == 1 {
        CoordPos::Inside
    } else {
        CoordPos::Outside
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, point, polygon};

    #[test]
    fn test_empty_poly() {
        let square_poly: Polygon<f64> = Polygon::new(LineString(vec![]), vec![]);
        assert_eq!(
            square_poly.coordinate_position(&Coordinate::zero()),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_simple_poly() {
        let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];

        let inside_coord = Coordinate { x: 1.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&inside_coord),
            CoordPos::Inside
        );

        let vertex_coord = Coordinate { x: 0.0, y: 0.0 };
        assert_eq!(
            square_poly.coordinate_position(&vertex_coord),
            CoordPos::OnBoundary
        );

        let boundary_coord = Coordinate { x: 0.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&boundary_coord),
            CoordPos::OnBoundary
        );

        let outside_coord = Coordinate { x: 5.0, y: 5.0 };
        assert_eq!(
            square_poly.coordinate_position(&outside_coord),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_poly_interior() {
        let poly = polygon![
            exterior: [
                (x: 11., y: 11.),
                (x: 20., y: 11.),
                (x: 20., y: 20.),
                (x: 11., y: 20.),
                (x: 11., y: 11.),
            ],
            interiors: [
                [
                    (x: 13., y: 13.),
                    (x: 13., y: 17.),
                    (x: 17., y: 17.),
                    (x: 17., y: 13.),
                    (x: 13., y: 13.),
                ]
            ],
        ];

        let inside_hole = Coordinate { x: 14.0, y: 14.0 };
        assert_eq!(poly.coordinate_position(&inside_hole), CoordPos::Outside);

        let outside_poly = Coordinate { x: 30.0, y: 30.0 };
        assert_eq!(poly.coordinate_position(&outside_poly), CoordPos::Outside);

        let on_outside_border = Coordinate { x: 20.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_outside_border),
            CoordPos::OnBoundary
        );

        let on_inside_border = Coordinate { x: 13.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_inside_border),
            CoordPos::OnBoundary
        );

        let inside_coord = Coordinate { x: 12.0, y: 12.0 };
        assert_eq!(poly.coordinate_position(&inside_coord), CoordPos::Inside);
    }

    #[test]
    fn test_simple_line() {
        use crate::point;
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 10.0, y: 10.0]);

        let start = Coordinate { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::OnBoundary);

        let end = Coordinate { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&end), CoordPos::OnBoundary);

        let interior = Coordinate { x: 5.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&interior), CoordPos::Inside);

        let outside = Coordinate { x: 6.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line() {
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 0.0, y: 0.0]);

        let start = Coordinate { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::Inside);

        let outside = Coordinate { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_point() {
        let p1 = point![x: 2.0, y: 0.0];

        let c1 = Coordinate { x: 2.0, y: 0.0 };
        let c2 = Coordinate { x: 3.0, y: 3.0 };

        assert_eq!(p1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(p1.coordinate_position(&c2), CoordPos::Outside);

        assert_eq!(c1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(c1.coordinate_position(&c2), CoordPos::Outside);
    }

    #[test]
    fn test_simple_line_string() {
        let line_string =
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 2.0, y: 0.0), (x: 3.0, y: 0.0)];

        let start = Coordinate::zero();
        assert_eq!(
            line_string.coordinate_position(&start),
            CoordPos::OnBoundary
        );

        let midpoint = Coordinate { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let vertex = Coordinate { x: 2.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&vertex), CoordPos::Inside);

        let end = Coordinate { x: 3.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&end), CoordPos::OnBoundary);

        let outside = Coordinate { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line_strings() {
        let line_string = line_string![(x: 0.0, y: 0.0), (x: 0.0, y: 0.0)];

        let start = Coordinate::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0)];

        let start = Coordinate::zero();
        assert_eq!(
            line_string.coordinate_position(&start),
            CoordPos::OnBoundary
        );
    }

    #[test]
    fn test_closed_line_string() {
        let line_string = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 2.0, y: 0.0), (x: 3.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];

        // sanity check
        assert!(line_string.is_closed());

        // closed line strings have no boundary
        let start = Coordinate::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let midpoint = Coordinate { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let outside = Coordinate { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_boundary_rule() {
        let multi_line_string = MultiLineString(vec![
            // first two lines have same start point but different end point
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)],
            line_string![(x: 0.0, y: 0.0), (x: -1.0, y: -1.0)],
            // third line has its own start point, but it's end touches the middle of first line
            line_string![(x: 0.0, y: 1.0), (x: 0.5, y: 0.5)],
            // fourth and fifth have independent start points, but both end at the middle of the
            // second line
            line_string![(x: 0.0, y: -1.0), (x: -0.5, y: -0.5)],
            line_string![(x: 0.0, y: -2.0), (x: -0.5, y: -0.5)],
        ]);

        let outside_of_all = Coordinate { x: 123.0, y: 123.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&outside_of_all),
            CoordPos::Outside
        );

        let end_of_one_line = Coordinate { x: -1.0, y: -1.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&end_of_one_line),
            CoordPos::OnBoundary
        );

        // in boundary of first and second, so considered *not* in the boundary by mod 2 rule
        let shared_start = Coordinate::zero();
        assert_eq!(
            multi_line_string.coordinate_position(&shared_start),
            CoordPos::Outside
        );

        // *in* the first line, on the boundary of the third line
        let one_end_plus_midpoint = Coordinate { x: 0.5, y: 0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&one_end_plus_midpoint),
            CoordPos::OnBoundary
        );

        // *in* the first line, on the *boundary* of the fourth and fifth line
        let two_ends_plus_midpoint = Coordinate { x: -0.5, y: -0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&two_ends_plus_midpoint),
            CoordPos::Inside
        );
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        assert_eq!(
            rect.coordinate_position(&Coordinate { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            rect.coordinate_position(&Coordinate { x: 0.0, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            rect.coordinate_position(&Coordinate { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_triangle() {
        let triangle = Triangle((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        assert_eq!(
            triangle.coordinate_position(&Coordinate { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            triangle.coordinate_position(&Coordinate { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            triangle.coordinate_position(&Coordinate { x: 2.49, y: 5.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_collection() {
        let triangle = Triangle((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        let collection = GeometryCollection(vec![triangle.into(), rect.into()]);

        //  outside of both
        assert_eq!(
            collection.coordinate_position(&Coordinate { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );

        // inside both
        assert_eq!(
            collection.coordinate_position(&Coordinate { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );

        // inside one, boundary of other
        assert_eq!(
            collection.coordinate_position(&Coordinate { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );

        //  boundary of both
        assert_eq!(
            collection.coordinate_position(&Coordinate { x: 5.0, y: 10.0 }),
            CoordPos::Outside
        );
    }
}
