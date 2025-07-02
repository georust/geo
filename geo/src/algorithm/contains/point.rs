use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::{CoordsIter, HasDimensions};
use crate::geometry::*;
use crate::{CoordNum, GeoFloat};

// ┌────────────────────────────────┐
// │ Implementations for Point      │
// └────────────────────────────────┘

impl<T> Contains<Coord<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        &self.0 == coord
    }
}

impl<T> Contains<Point<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            // degenerate line is a point
            line.start == self.0
        } else {
            false
        }
    }
}

impl<T> Contains<LineString<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, line_string: &LineString<T>) -> bool {
        if line_string.is_empty() {
            return false;
        }
        // only a degenerate LineString could be within a point
        line_string.coords().all(|c| c == &self.0)
    }
}

impl<T> Contains<Polygon<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, polygon: &Polygon<T>) -> bool {
        if polygon.is_empty() {
            return false;
        }
        // only a degenerate Polygon could be within a point
        polygon.coords_iter().all(|coord| coord == self.0)
    }
}

impl<T> Contains<MultiPoint<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_point: &MultiPoint<T>) -> bool {
        if multi_point.is_empty() {
            return false;
        }
        multi_point.iter().all(|point| self.contains(point))
    }
}

impl<T> Contains<MultiLineString<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_line_string: &MultiLineString<T>) -> bool {
        if multi_line_string.is_empty() {
            return false;
        }
        // only a degenerate MultiLineString could be within a point
        multi_line_string
            .iter()
            .all(|line_string| self.contains(line_string))
    }
}

impl<T> Contains<MultiPolygon<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_polygon: &MultiPolygon<T>) -> bool {
        if multi_polygon.is_empty() {
            return false;
        }
        // only a degenerate MultiPolygon could be within a point
        multi_polygon.iter().all(|polygon| self.contains(polygon))
    }
}

impl<T> Contains<GeometryCollection<T>> for Point<T>
where
    T: GeoFloat,
{
    fn contains(&self, geometry_collection: &GeometryCollection<T>) -> bool {
        if geometry_collection.is_empty() {
            return false;
        }
        geometry_collection
            .iter()
            .all(|geometry| self.contains(geometry))
    }
}

impl<T> Contains<Rect<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, rect: &Rect<T>) -> bool {
        // only a degenerate Rect could be within a point
        rect.min() == rect.max() && rect.min() == self.0
    }
}

impl<T> Contains<Triangle<T>> for Point<T>
where
    T: CoordNum,
{
    fn contains(&self, triangle: &Triangle<T>) -> bool {
        // only a degenerate Triangle could be within a point
        triangle.0 == triangle.1 && triangle.0 == triangle.2 && triangle.0 == self.0
    }
}

impl_contains_geometry_for!(Point<T>);

// ┌────────────────────────────────┐
// │ Implementations for MultiPoint │
// └────────────────────────────────┘

impl_contains_from_relate!(MultiPoint<T>, [Line<T>, LineString<T>, Polygon<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);

impl<T> Contains<Coord<T>> for MultiPoint<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|c| &c.0 == coord)
    }
}

impl<T> Contains<Point<T>> for MultiPoint<T>
where
    T: CoordNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.iter().any(|c| c == point)
    }
}

impl<T> Contains<MultiPoint<T>> for MultiPoint<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_point: &MultiPoint<T>) -> bool {
        // sort both collections by x then y
        // then double pointer our way up the sorted arrays

        if self.0.is_empty() {
            return false;
        }
        if multi_point.0.is_empty() {
            return false;
        }

        let mut self_order = self.0.clone();
        self_order.sort_by(cmp_pts);

        let mut other_order = multi_point.0.clone();
        other_order.sort_by(cmp_pts);

        let mut self_iter = self_order.iter().peekable();
        let mut other_iter = other_order.iter().peekable();

        loop {
            // other has been exhausted
            if other_iter.peek().is_none() {
                return true;
            }
            // self has been exhausted but other has not been exhausted
            if self_iter.peek().is_none() {
                return false;
            }

            match cmp_pts(self_iter.peek().unwrap(), other_iter.peek().unwrap()) {
                std::cmp::Ordering::Equal => {
                    // other only ensures that we don't step past duplicate other points
                    other_iter.next();
                }
                std::cmp::Ordering::Less => {
                    self_iter.next();
                }
                std::cmp::Ordering::Greater => {
                    return false;
                }
            }
        }
    }
}

// used for sorting points in multipoint
fn cmp_pts<T: CoordNum>(a: &Point<T>, b: &Point<T>) -> std::cmp::Ordering {
    let x_order = a.x().partial_cmp(&b.x());
    match x_order {
        Some(std::cmp::Ordering::Equal) => {
            let y_order = a.y().partial_cmp(&b.y());
            match y_order {
                Some(std::cmp::Ordering::Equal) => std::cmp::Ordering::Equal,
                Some(std::cmp::Ordering::Less) => std::cmp::Ordering::Less,
                Some(std::cmp::Ordering::Greater) => std::cmp::Ordering::Greater,
                None => std::cmp::Ordering::Equal,
            }
        }
        Some(std::cmp::Ordering::Less) => std::cmp::Ordering::Less,
        Some(std::cmp::Ordering::Greater) => std::cmp::Ordering::Greater,
        None => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, point, MultiPoint, Relate};

    #[test]
    /**
     * tests for empty multipoint
     * behaviour follows `Relate` Trait
     */
    fn test_empty_multipoint_contains_multipoint() {
        let empty: MultiPoint<f64> = MultiPoint::new(Vec::new());
        let non_empty: MultiPoint<f64> = MultiPoint::new(vec![point! {x: 0.0, y: 0.0}]);

        // empty multipoint does not contains empty multipoint
        assert!(!empty.contains(&non_empty));
        assert!(!empty.relate(&non_empty).is_contains());

        // non-empty multipoint does not contain empty multipoint
        assert!(!non_empty.contains(&empty));
        assert!(!non_empty.relate(&empty).is_contains());

        // empty multipoint does not contain empty multipoint
        assert!(!empty.contains(&empty));
        assert!(!empty.relate(&empty).is_contains());
    }

    #[test]
    fn test_multipoint_contains_multipoint() {
        let pt_a = coord! {x: 0., y: 0.};
        let pt_b = coord! {x: 10., y: 10.};
        let pt_c = coord! {x: 20., y: 20.};
        let pt_d = coord! {x: 30., y: 30.};

        let mp_a = MultiPoint::from(vec![pt_a]);
        let mp_bc = MultiPoint::from(vec![pt_a, pt_b]);
        let mp_abc = MultiPoint::from(vec![pt_a, pt_b, pt_c]);
        let mp_bcd = MultiPoint::from(vec![pt_b, pt_c, pt_d]);

        // multipoint contains itself
        assert!(mp_a.contains(&mp_a));
        assert!(mp_bc.contains(&mp_bc));
        assert!(mp_abc.contains(&mp_abc));
        assert!(mp_bcd.contains(&mp_bcd));

        // multipoint contains subsets
        assert!(mp_abc.contains(&mp_a));
        assert!(mp_abc.contains(&mp_bc));

        // overlapping multipoints do not contain each other
        assert!(!mp_abc.contains(&mp_bcd));
    }
}
