use std::cmp::Ordering;

use crate::geometry::*;
use crate::intersects::{point_in_rect, value_in_between};
use crate::kernels::*;
use crate::{BoundingRect, HasDimensions, Intersects};
use crate::{GeoNum, GeometryCow};
use geo_traits::to_geo::ToGeoCoord;
use geo_traits_ext::*;

/// The position of a `Coord` relative to a `Geometry`
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CoordPos {
    OnBoundary,
    Inside,
    Outside,
}

/// Determine whether a `Coord` lies inside, outside, or on the boundary of a geometry.
///
/// # Examples
///
/// ```rust
/// use geo::{polygon, coord};
/// use geo::coordinate_position::{CoordinatePosition, CoordPos};
///
/// let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];
///
/// let inside_coord = coord! { x: 1.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&inside_coord), CoordPos::Inside);
///
/// let boundary_coord = coord! { x: 0.0, y: 1.0 };
/// assert_eq!(square_poly.coordinate_position(&boundary_coord), CoordPos::OnBoundary);
///
/// let outside_coord = coord! { x: 5.0, y: 5.0 };
/// assert_eq!(square_poly.coordinate_position(&outside_coord), CoordPos::Outside);
/// ```
pub trait CoordinatePosition {
    type Scalar: GeoNum;
    fn coordinate_position(&self, coord: &Coord<Self::Scalar>) -> CoordPos {
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
        coord: &Coord<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    );
}

impl<G> CoordinatePosition for G
where
    G: GeoTraitExtWithTypeTag,
    G: CoordinatePositionTrait<G::Tag>,
{
    type Scalar = G::T;

    fn calculate_coordinate_position(
        &self,
        coord: &Coord<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        self.calculate_coordinate_position_trait(coord, is_inside, boundary_count);
    }
}

pub trait CoordinatePositionTrait<GT: GeoTypeTag> {
    type T: GeoNum;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<Self::T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    );
}

impl<T, C> CoordinatePositionTrait<CoordTag> for C
where
    T: GeoNum,
    C: CoordTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if &self.to_coord() == coord {
            *is_inside = true;
        }
    }
}

impl<T, P> CoordinatePositionTrait<PointTag> for P
where
    T: GeoNum,
    P: PointTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if let Some(point_coord) = self.coord() {
            if &point_coord.to_coord() == coord {
                *is_inside = true;
            }
        }
    }
}

impl<T, L> CoordinatePositionTrait<LineTag> for L
where
    T: GeoNum,
    L: LineTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let start = self.start_ext().to_coord();
        let end = self.end_ext().to_coord();

        // degenerate line is a point
        if start == end {
            self.start_ext()
                .calculate_coordinate_position(coord, is_inside, boundary_count);
            return;
        }

        if coord == &start || coord == &end {
            *boundary_count += 1;
        } else if self.intersects(coord) {
            *is_inside = true;
        }
    }
}

impl<T, LS> CoordinatePositionTrait<LineStringTag> for LS
where
    T: GeoNum,
    LS: LineStringTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let num_coords = self.num_coords();
        if num_coords < 2 {
            debug_assert!(false, "invalid line string with less than 2 coords");
            return;
        }

        if num_coords == 2 {
            // line string with two coords is just a line
            unsafe {
                let start = self.coord_unchecked_ext(0).to_coord();
                let end = self.coord_unchecked_ext(1).to_coord();
                Line::new(start, end).calculate_coordinate_position(
                    coord,
                    is_inside,
                    boundary_count,
                );
            }
            return;
        }

        // optimization: return early if there's no chance of an intersection
        // since bounding rect is not empty, we can safely `unwrap`.
        if !self.bounding_rect().unwrap().intersects(coord) {
            return;
        }

        // A closed linestring has no boundary, per SFS
        if !self.is_closed() {
            // since we have at least two coords, first and last will exist
            unsafe {
                let first = self.coord_unchecked_ext(0).to_coord();
                let last = self.coord_unchecked_ext(num_coords - 1).to_coord();
                if coord == &first || coord == &last {
                    *boundary_count += 1;
                    return;
                }
            }
        }

        if self.intersects(coord) {
            // We've already checked for "Boundary" condition, so if there's an intersection at
            // this point, coord must be on the interior
            *is_inside = true
        }
    }
}

impl<T, TT> CoordinatePositionTrait<TriangleTag> for TT
where
    T: GeoNum,
    TT: TriangleTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        *is_inside = self
            .to_lines()
            .map(|l| {
                let orientation = T::Ker::orient2d(l.start, l.end, *coord);
                if orientation == Orientation::Collinear
                    && point_in_rect(*coord, l.start, l.end)
                    && coord.x != l.end.x
                {
                    *boundary_count += 1;
                }
                orientation
            })
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear);
    }
}

impl<T, R> CoordinatePositionTrait<RectTag> for R
where
    T: GeoNum,
    R: RectTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let mut boundary = false;
        let min = self.min().to_coord();

        match coord.x.partial_cmp(&min.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match coord.y.partial_cmp(&min.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }

        let max = self.max().to_coord();

        match max.x.partial_cmp(&coord.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match max.y.partial_cmp(&coord.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }

        if boundary {
            *boundary_count += 1;
        } else {
            *is_inside = true;
        }
    }
}

impl<T, MP> CoordinatePositionTrait<MultiPointTag> for MP
where
    T: GeoNum,
    MP: MultiPointTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self
            .points_ext()
            .any(|p| p.coord_ext().is_some_and(|c| &c.to_coord() == coord))
        {
            *is_inside = true;
        }
    }
}

impl<T, P> CoordinatePositionTrait<PolygonTag> for P
where
    T: GeoNum,
    P: PolygonTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let Some(exterior) = self.exterior_ext() else {
            return;
        };

        if self.is_empty() {
            return;
        }

        match coord_pos_relative_to_ring(*coord, &exterior) {
            CoordPos::Outside => {}
            CoordPos::OnBoundary => {
                *boundary_count += 1;
            }
            CoordPos::Inside => {
                for hole in self.interiors_ext() {
                    match coord_pos_relative_to_ring(*coord, &hole) {
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

impl<T, MLS> CoordinatePositionTrait<MultiLineStringTag> for MLS
where
    T: GeoNum,
    MLS: MultiLineStringTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for line_string in self.line_strings_ext() {
            line_string.calculate_coordinate_position_trait(coord, is_inside, boundary_count);
        }
    }
}

impl<T, MP> CoordinatePositionTrait<MultiPolygonTag> for MP
where
    T: GeoNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for polygon in self.polygons_ext() {
            polygon.calculate_coordinate_position_trait(coord, is_inside, boundary_count);
        }
    }
}

impl<T, GC> CoordinatePositionTrait<GeometryCollectionTag> for GC
where
    T: GeoNum,
    GC: GeometryCollectionTraitExt<T = T>,
{
    type T = T;

    fn calculate_coordinate_position_trait(
        &self,
        coord: &Coord<T>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for geometry in self.geometries_ext() {
            geometry.calculate_coordinate_position_trait(coord, is_inside, boundary_count);
        }
    }
}

impl<T, G> CoordinatePositionTrait<GeometryTag> for G
where
    T: GeoNum,
    G: GeometryTraitExt<T = T>,
{
    type T = T;

    crate::geometry_trait_ext_delegate_impl! {
        fn calculate_coordinate_position_trait(
            &self,
            coord: &Coord<T>,
            is_inside: &mut bool,
            boundary_count: &mut usize) -> ();
    }
}

impl<T: GeoNum> CoordinatePosition for GeometryCow<'_, T> {
    type Scalar = T;

    crate::geometry_cow_delegate_impl! {
        fn calculate_coordinate_position(
            &self,
            coord: &Coord<T>,
            is_inside: &mut bool,
            boundary_count: &mut usize) -> ();
    }
}

/// Calculate the position of a `Coord` relative to a
/// closed `LineString`.
pub fn coord_pos_relative_to_ring<T, LS>(coord: Coord<T>, linestring: &LS) -> CoordPos
where
    T: GeoNum,
    LS: LineStringTraitExt<T = T>,
{
    debug_assert!(linestring.is_closed());

    // LineString without points
    if linestring.num_coords() == 0 {
        return CoordPos::Outside;
    }
    if linestring.num_coords() == 1 {
        // If LineString has one point, it will not generate
        // any lines.  So, we handle this edge case separately.
        return if coord == unsafe { linestring.coord_unchecked_ext(0).to_coord() } {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    // Use winding number algorithm with on boundary short-cicuit
    // See: https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm
    let mut winding_number = 0;
    for line in linestring.lines() {
        // Edge Crossing Rules:
        //   1. an upward edge includes its starting endpoint, and excludes its final endpoint;
        //   2. a downward edge excludes its starting endpoint, and includes its final endpoint;
        //   3. horizontal edges are excluded
        //   4. the edge-ray intersection point must be strictly right of the coord.
        if line.start.y <= coord.y {
            if line.end.y >= coord.y {
                let o = T::Ker::orient2d(line.start, line.end, coord);
                if o == Orientation::CounterClockwise && line.end.y != coord.y {
                    winding_number += 1
                } else if o == Orientation::Collinear
                    && value_in_between(coord.x, line.start.x, line.end.x)
                {
                    return CoordPos::OnBoundary;
                }
            };
        } else if line.end.y <= coord.y {
            let o = T::Ker::orient2d(line.start, line.end, coord);
            if o == Orientation::Clockwise {
                winding_number -= 1
            } else if o == Orientation::Collinear
                && value_in_between(coord.x, line.start.x, line.end.x)
            {
                return CoordPos::OnBoundary;
            }
        }
    }
    if winding_number == 0 {
        CoordPos::Outside
    } else {
        CoordPos::Inside
    }
}

#[cfg(test)]
mod test {
    use geo_types::coord;

    use super::*;
    use crate::{line_string, point, polygon};

    #[test]
    fn test_empty_poly() {
        let square_poly: Polygon<f64> = Polygon::new(LineString::new(vec![]), vec![]);
        assert_eq!(
            square_poly.coordinate_position(&Coord::zero()),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_simple_poly() {
        let square_poly = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 2.0), (x: 0.0, y: 2.0), (x: 0.0, y: 0.0)];

        let inside_coord = coord! { x: 1.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&inside_coord),
            CoordPos::Inside
        );

        let vertex_coord = coord! { x: 0.0, y: 0.0 };
        assert_eq!(
            square_poly.coordinate_position(&vertex_coord),
            CoordPos::OnBoundary
        );

        let boundary_coord = coord! { x: 0.0, y: 1.0 };
        assert_eq!(
            square_poly.coordinate_position(&boundary_coord),
            CoordPos::OnBoundary
        );

        let outside_coord = coord! { x: 5.0, y: 5.0 };
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

        let inside_hole = coord! { x: 14.0, y: 14.0 };
        assert_eq!(poly.coordinate_position(&inside_hole), CoordPos::Outside);

        let outside_poly = coord! { x: 30.0, y: 30.0 };
        assert_eq!(poly.coordinate_position(&outside_poly), CoordPos::Outside);

        let on_outside_border = coord! { x: 20.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_outside_border),
            CoordPos::OnBoundary
        );

        let on_inside_border = coord! { x: 13.0, y: 15.0 };
        assert_eq!(
            poly.coordinate_position(&on_inside_border),
            CoordPos::OnBoundary
        );

        let inside_coord = coord! { x: 12.0, y: 12.0 };
        assert_eq!(poly.coordinate_position(&inside_coord), CoordPos::Inside);
    }

    #[test]
    fn test_simple_line() {
        use crate::point;
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 10.0, y: 10.0]);

        let start = coord! { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::OnBoundary);

        let end = coord! { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&end), CoordPos::OnBoundary);

        let interior = coord! { x: 5.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&interior), CoordPos::Inside);

        let outside = coord! { x: 6.0, y: 5.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line() {
        let line = Line::new(point![x: 0.0, y: 0.0], point![x: 0.0, y: 0.0]);

        let start = coord! { x: 0.0, y: 0.0 };
        assert_eq!(line.coordinate_position(&start), CoordPos::Inside);

        let outside = coord! { x: 10.0, y: 10.0 };
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_point() {
        let p1 = point![x: 2.0, y: 0.0];

        let c1 = coord! { x: 2.0, y: 0.0 };
        let c2 = coord! { x: 3.0, y: 3.0 };

        assert_eq!(p1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(p1.coordinate_position(&c2), CoordPos::Outside);

        assert_eq!(c1.coordinate_position(&c1), CoordPos::Inside);
        assert_eq!(c1.coordinate_position(&c2), CoordPos::Outside);
    }

    #[test]
    fn test_simple_line_string() {
        let line_string =
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 2.0, y: 0.0), (x: 3.0, y: 0.0)];

        let start = Coord::zero();
        assert_eq!(
            line_string.coordinate_position(&start),
            CoordPos::OnBoundary
        );

        let midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let vertex = coord! { x: 2.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&vertex), CoordPos::Inside);

        let end = coord! { x: 3.0, y: 0.0 };
        assert_eq!(line_string.coordinate_position(&end), CoordPos::OnBoundary);

        let outside = coord! { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_degenerate_line_strings() {
        let line_string = line_string![(x: 0.0, y: 0.0), (x: 0.0, y: 0.0)];

        let start = Coord::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0)];

        let start = Coord::zero();
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
        let start = Coord::zero();
        assert_eq!(line_string.coordinate_position(&start), CoordPos::Inside);

        let midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(line_string.coordinate_position(&midpoint), CoordPos::Inside);

        let outside = coord! { x: 3.0, y: 1.0 };
        assert_eq!(line_string.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_boundary_rule() {
        let multi_line_string = MultiLineString::new(vec![
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

        let outside_of_all = coord! { x: 123.0, y: 123.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&outside_of_all),
            CoordPos::Outside
        );

        let end_of_one_line = coord! { x: -1.0, y: -1.0 };
        assert_eq!(
            multi_line_string.coordinate_position(&end_of_one_line),
            CoordPos::OnBoundary
        );

        // in boundary of first and second, so considered *not* in the boundary by mod 2 rule
        let shared_start = Coord::zero();
        assert_eq!(
            multi_line_string.coordinate_position(&shared_start),
            CoordPos::Outside
        );

        // *in* the first line, on the boundary of the third line
        let one_end_plus_midpoint = coord! { x: 0.5, y: 0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&one_end_plus_midpoint),
            CoordPos::OnBoundary
        );

        // *in* the first line, on the *boundary* of the fourth and fifth line
        let two_ends_plus_midpoint = coord! { x: -0.5, y: -0.5 };
        assert_eq!(
            multi_line_string.coordinate_position(&two_ends_plus_midpoint),
            CoordPos::Inside
        );
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        assert_eq!(
            rect.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            rect.coordinate_position(&coord! { x: 0.0, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            rect.coordinate_position(&coord! { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_triangle() {
        let triangle = Triangle::new((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );
        assert_eq!(
            triangle.coordinate_position(&coord! { x: 2.49, y: 5.0 }),
            CoordPos::Outside
        );
    }

    #[test]
    fn test_collection() {
        let triangle = Triangle::new((0.0, 0.0).into(), (5.0, 10.0).into(), (10.0, 0.0).into());
        let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
        let collection = GeometryCollection::new_from(vec![triangle.into(), rect.into()]);

        //  outside of both
        assert_eq!(
            collection.coordinate_position(&coord! { x: 15.0, y: 15.0 }),
            CoordPos::Outside
        );

        // inside both
        assert_eq!(
            collection.coordinate_position(&coord! { x: 5.0, y: 5.0 }),
            CoordPos::Inside
        );

        // inside one, boundary of other
        assert_eq!(
            collection.coordinate_position(&coord! { x: 2.5, y: 5.0 }),
            CoordPos::OnBoundary
        );

        //  boundary of both
        assert_eq!(
            collection.coordinate_position(&coord! { x: 5.0, y: 10.0 }),
            CoordPos::Outside
        );
    }
}
