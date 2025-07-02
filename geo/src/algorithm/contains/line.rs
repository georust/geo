use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::Intersects;
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

impl<T> Contains<Coord<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        if self.start == self.end {
            &self.start == coord
        } else {
            coord != &self.start && coord != &self.end && self.intersects(coord)
        }
    }
}

impl<T> Contains<Point<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            self.contains(&line.start)
        } else {
            self.intersects(&line.start) && self.intersects(&line.end)
        }
    }
}

impl<T> Contains<LineString<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // Empty linestring has no interior, and not
        // contained in anything.
        if linestring.0.is_empty() {
            return false;
        }

        // The interior of the linestring should have some
        // intersection with the interior of self. Two cases
        // arise:
        //
        // 1. There are at least two distinct points in the
        // linestring. Then, if both intersect, the interior
        // between these two must have non-empty intersection.
        //
        // 2. Otherwise, all the points on the linestring
        // are the same. In this case, the interior is this
        // specific point, and it should be contained in the
        // line.
        let first = linestring.0.first().unwrap();
        let mut all_equal = true;

        // If all the vertices of the linestring intersect
        // self, then the interior or boundary of the
        // linestring cannot have non-empty intersection
        // with the exterior.
        let all_intersects = linestring.0.iter().all(|c| {
            if c != first {
                all_equal = false;
            }
            self.intersects(c)
        });

        all_intersects && (!all_equal || self.contains(first))
    }
}

impl<T> Contains<MultiPoint<T>> for Line<T>
where
    T: GeoNum,
{
    fn contains(&self, multi_point: &MultiPoint<T>) -> bool {
        // at least one point must not be equal to one of the vertices

        multi_point.iter().any(|point| self.contains(&point.0))
            && multi_point.iter().all(|point| self.intersects(&point.0))
    }
}
impl_contains_from_relate!(Line<T>, [Polygon<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Line<T>);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, MultiPoint, Relate};

    #[test]
    fn test_line_contains_empty_multipoint() {
        let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
        let empty: MultiPoint<f64> = MultiPoint::new(Vec::new());

        assert!(!line.contains(&empty));
        assert!(!line.relate(&empty).is_contains());
    }

    #[test]
    fn test_line_contains_multipoint() {
        let start = coord! {x: 0., y: 0.};
        let mid = coord! {x: 50., y: 50.};
        let end = coord! {x: 100., y: 100.};
        let out = coord! {x: 101., y: 101.};

        let line = Line::new(start, end);

        let mp_ends = MultiPoint::from(vec![start, end]);
        let mp_within = MultiPoint::from(vec![mid]);
        let mp_merged = MultiPoint::from(vec![start, mid, end]);

        let mp_out = MultiPoint::from(vec![out]);
        let mp_all = MultiPoint::from(vec![start, mid, end, out]);

        // false if all points lie on the boundary of the line (start and end points)
        assert!(!line.contains(&mp_ends));

        // at least one point must be
        assert!(line.contains(&mp_within));
        assert!(line.contains(&mp_merged));

        // return false if any point is not on the line
        assert!(!line.contains(&mp_out));
        assert!(!line.contains(&mp_all));

        // multipoint containing duplicates
        let start_dupe = MultiPoint::from(vec![start, start]);
        let start_dupe_within = MultiPoint::from(vec![start, start, mid]);

        assert!(!line.contains(&start_dupe));
        assert!(line.contains(&start_dupe_within));
    }
}
