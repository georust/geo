use geo_traits::to_geo::ToGeoCoord;
use geo_traits_ext::{CoordTag, CoordTraitExt, LineTag, LineTraitExt, PointTag, PointTraitExt, TriangleTag, TriangleTraitExt};

use super::{point_in_rect, Intersects, IntersectsTrait};
use crate::*;

impl<T> Intersects<Coord<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        // First we check if the point is collinear with the line.
        T::Ker::orient2d(self.start, self.end, *rhs) == Orientation::Collinear
        // In addition, the point must have _both_ coordinates
        // within the start and end bounds.
            && point_in_rect(*rhs, self.start, self.end)
    }
}
symmetric_intersects_impl!(Coord<T>, Line<T>);
symmetric_intersects_impl!(Line<T>, Point<T>);

impl<T> Intersects<Line<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        // Special case: self is equiv. to a point.
        if self.start == self.end {
            return line.intersects(&self.start);
        }

        // Precondition: start and end are distinct.

        // Check if orientation of rhs.{start,end} are different
        // with respect to self.{start,end}.
        let check_1_1 = T::Ker::orient2d(self.start, self.end, line.start);
        let check_1_2 = T::Ker::orient2d(self.start, self.end, line.end);

        if check_1_1 != check_1_2 {
            // Since the checks are different,
            // rhs.{start,end} are distinct, and rhs is not
            // collinear with self. Thus, there is exactly
            // one point on the infinite extensions of rhs,
            // that is collinear with self.

            // By continuity, this point is not on the
            // exterior of rhs. Now, check the same with
            // self, rhs swapped.

            let check_2_1 = T::Ker::orient2d(line.start, line.end, self.start);
            let check_2_2 = T::Ker::orient2d(line.start, line.end, self.end);

            // By similar argument, there is (exactly) one
            // point on self, collinear with rhs. Thus,
            // those two have to be same, and lies (interior
            // or boundary, but not exterior) on both lines.
            check_2_1 != check_2_2
        } else if check_1_1 == Orientation::Collinear {
            // Special case: collinear line segments.

            // Equivalent to 4 point-line intersection
            // checks, but removes the calls to the kernel
            // predicates.
            point_in_rect(line.start, self.start, self.end)
                || point_in_rect(line.end, self.start, self.end)
                || point_in_rect(self.end, line.start, line.end)
                || point_in_rect(self.end, line.start, line.end)
        } else {
            false
        }
    }
}

impl<T> Intersects<Triangle<T>> for Line<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        self.intersects(&rhs.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T>, Line<T>);

///// New Code

impl<T, LHS, RHS> IntersectsTrait<LineTag, CoordTag, RHS> for LHS
where
    T: GeoNum,
    LHS: LineTraitExt<T = T>,
    RHS: CoordTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        let start = self.start_ext().to_coord();
        let end = self.end_ext().to_coord();
        let rhs = rhs.to_coord();

        // First we check if the point is collinear with the line.
        T::Ker::orient2d(start, end, rhs) == Orientation::Collinear
        // In addition, the point must have _both_ coordinates
        // within the start and end bounds.
            && point_in_rect(rhs, start, end)
    }
}

symmetric_intersects_trait_impl!(GeoNum, CoordTraitExt, CoordTag, LineTraitExt, LineTag);
symmetric_intersects_trait_impl!(GeoNum, LineTraitExt, LineTag, PointTraitExt, PointTag);

impl<T, LHS, RHS> IntersectsTrait<LineTag, LineTag, RHS> for LHS
where
    T: GeoNum,
    LHS: LineTraitExt<T = T>,
    RHS: LineTraitExt<T = T>,
{
    fn intersects_trait(&self, line: &RHS) -> bool {
        let start_ext = self.start_ext();
        let self_start = start_ext.to_coord();
        let self_end = self.end_ext().to_coord();
        let line_start = line.start_ext().to_coord();
        let line_end = line.end_ext().to_coord();

        // Special case: self is equiv. to a point.
        if self_start == self_end {
            return line.intersects_trait(&start_ext);
        }

        // Precondition: start and end are distinct.

        // Check if orientation of rhs.{start,end} are different
        // with respect to self.{start,end}.
        let check_1_1 = T::Ker::orient2d(self_start, self_end, line_start);
        let check_1_2 = T::Ker::orient2d(self_start, self_end, line_end);

        if check_1_1 != check_1_2 {
            // Since the checks are different,
            // rhs.{start,end} are distinct, and rhs is not
            // collinear with self. Thus, there is exactly
            // one point on the infinite extensions of rhs,
            // that is collinear with self.

            // By continuity, this point is not on the
            // exterior of rhs. Now, check the same with
            // self, rhs swapped.

            let check_2_1 = T::Ker::orient2d(line_start, line_end, self_start);
            let check_2_2 = T::Ker::orient2d(line_start, line_end, self_end);

            // By similar argument, there is (exactly) one
            // point on self, collinear with rhs. Thus,
            // those two have to be same, and lies (interior
            // or boundary, but not exterior) on both lines.
            check_2_1 != check_2_2
        } else if check_1_1 == Orientation::Collinear {
            // Special case: collinear line segments.

            // Equivalent to 4 point-line intersection
            // checks, but removes the calls to the kernel
            // predicates.
            point_in_rect(line_start, self_start, self_end)
                || point_in_rect(line_end, self_start, self_end)
                || point_in_rect(self_end, line_start, line_end)
                || point_in_rect(self_end, line_start, line_end)
        } else {
            false
        }
    }
}

impl<T, LHS, RHS> IntersectsTrait<LineTag, TriangleTag, RHS> for LHS
where
    T: GeoNum,
    LHS: LineTraitExt<T = T>,
    RHS: TriangleTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        // self.intersects_trait(line)
        let polygon = rhs.to_polygon();
        self.intersects_trait(&polygon)
    }
}

symmetric_intersects_trait_impl!(GeoNum, TriangleTraitExt, TriangleTag, LineTraitExt, LineTag);

fn test_line_intersects_line_general<L1, L2>(l1: &L1, l2: &L2) -> bool
where 
    L1: LineTraitExt<T = f64>,
    L2: LineTraitExt<T = f64>,
{
    l1.intersects_trait(l2)
}

#[test]
fn test_line_intersects_line() {
    let l1 = Line::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
    let l2 = Line::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0));
    let l3 = Line::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0));
    let l4 = Line::new(Point::new(10.0, 10.0), Point::new(11.0, 11.0));
    assert!(test_line_intersects_line_general(&l1, &l2));
    assert!(test_line_intersects_line_general(&l1, &l3));
    assert!(test_line_intersects_line_general(&l2, &l3));
    assert_ne!(test_line_intersects_line_general(&l1, &l4), true);
    assert_ne!(test_line_intersects_line_general(&l2, &l4), true);
    assert_ne!(test_line_intersects_line_general(&l3, &l4), true);
}
