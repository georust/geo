// To implement RStar’s traits in the geo-types crates, we need to access to a
// few geospatial algorithms, which are included in this hidden module. This
// hidden module is public so the geo crate can reuse these algorithms to
// prevent duplication. These functions are _not_ meant for public consumption.

use crate::{Coord, CoordFloat, CoordNum, Line, LineString, Point, Rect};

pub fn line_string_bounding_rect<T>(line_string: &LineString<T>) -> Option<Rect<T>>
where
    T: CoordNum,
{
    get_bounding_rect(&line_string.0)
}

pub fn line_bounding_rect<T>(line: Line<T>) -> Rect<T>
where
    T: CoordNum,
{
    Rect::new(line.start, line.end)
}

pub fn get_bounding_rect<I, C, T>(collection: I) -> Option<Rect<T>>
where
    T: CoordNum,
    C: AsRef<Coord<T>>,
    I: IntoIterator<Item = C>,
{
    let mut iter = collection.into_iter();
    let first = *iter.next()?.as_ref();
    // Four independent accumulators, so that consecutive comparisons do not wait for each
    // other. Each accumulator holds the minimum and maximum corners.
    let mut acc = [(first, first); 4];
    'outer: loop {
        for (min, max) in &mut acc {
            let Some(pnt) = iter.next() else {
                break 'outer;
            };
            update_min_max(min, max, *pnt.as_ref());
        }
    }
    let [(mut min, mut max), rest @ ..] = acc;
    for (other_min, other_max) in rest {
        update_min_max(&mut min, &mut max, other_min);
        update_min_max(&mut min, &mut max, other_max);
    }
    Some(Rect::new(min, max))
}

/// Extend the `min` and `max` corners to include `pnt`. A NaN component does not change a
/// corner.
#[inline(always)]
fn update_min_max<T: CoordNum>(min: &mut Coord<T>, max: &mut Coord<T>, pnt: Coord<T>) {
    if pnt.x < min.x {
        min.x = pnt.x;
    }
    if pnt.y < min.y {
        min.y = pnt.y;
    }
    if pnt.x > max.x {
        max.x = pnt.x;
    }
    if pnt.y > max.y {
        max.y = pnt.y;
    }
}

pub fn line_segment_distance<T, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    let point = point.into();
    let start = start.into();
    let end = end.into();

    if start == end {
        return line_euclidean_length(Line::new(point, start));
    }
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let (px, py) = (point.x - start.x, point.y - start.y);

    // The squared segment length can overflow or underflow, which makes the projection
    // parameter NaN or wrong. Use the raw differences only where it does neither.
    let d_squared = dx * dx + dy * dy;
    if !(d_squared.is_finite() && d_squared >= T::min_positive_value()) {
        return scaled_line_segment_distance(point, start, end);
    }
    let r = (px * dx + py * dy) / d_squared;
    if r <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new(point, end));
    }
    ((py * dx - px * dy) / d_squared).abs() * dx.hypot(dy)
}

/// The distance from `point` to a segment of distinct endpoints whose squared length
/// overflows or underflows.
#[cold]
#[inline(never)]
fn scaled_line_segment_distance<T>(point: Coord<T>, start: Coord<T>, end: Coord<T>) -> T
where
    T: CoordFloat,
{
    let two = T::one() + T::one();
    let (mut dx, mut dy) = (end.x - start.x, end.y - start.y);
    let (mut px, mut py) = (point.x - start.x, point.y - start.y);
    let mut segment_scale = dx.abs().max(dy.abs());
    let mut point_scale = px.abs().max(py.abs());

    // The difference of two finite coordinates can overflow. Halve the coordinates before
    // the subtraction, and keep the factor of 2 separately.
    let mut segment_factor = T::one();
    let mut point_factor = T::one();
    if !segment_scale.is_finite() {
        dx = end.x / two - start.x / two;
        dy = end.y / two - start.y / two;
        segment_scale = dx.abs().max(dy.abs());
        segment_factor = two;
    }
    if !point_scale.is_finite() {
        px = point.x / two - start.x / two;
        py = point.y / two - start.y / two;
        point_scale = px.abs().max(py.abs());
        point_factor = two;
    }
    if point_scale == T::zero() {
        // The point is the start of the segment.
        return T::zero();
    }
    // Scale each vector by its largest component. The two vectors get independent scales:
    // one shared scale underflows the squared segment length when a distant point meets a
    // short segment.
    dx = dx / segment_scale;
    dy = dy / segment_scale;
    px = px / point_scale;
    py = py / point_scale;
    let d_squared = dx * dx + dy * dy;
    let dot = px * dx + py * dy;
    if dot <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    // This tests dot >= d_squared for the unscaled vectors. Where the scales are far apart,
    // their ratio overflows or underflows, and the test still selects the correct branch.
    let scale_ratio = point_scale / segment_scale * (point_factor / segment_factor);
    if scale_ratio * dot >= d_squared {
        return line_euclidean_length(Line::new(point, end));
    }
    point_factor * (point_scale * ((py * dx - px * dy) / d_squared).abs() * dx.hypot(dy))
}

pub fn line_segment_distance_squared<T, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    let distance = line_segment_distance(point, start, end);
    distance * distance
}

pub fn line_euclidean_length<T>(line: Line<T>) -> T
where
    T: CoordFloat,
{
    line.dx().hypot(line.dy())
}

pub fn point_line_string_euclidean_distance<T>(p: Point<T>, l: &LineString<T>) -> T
where
    T: CoordFloat,
{
    // No need to continue if the point is on the LineString, or it's empty
    if line_string_contains_point(l, p) || l.0.is_empty() {
        return T::zero();
    }
    l.lines()
        .map(|line| line_segment_distance(p.0, line.start, line.end))
        .fold(T::max_value(), |accum, val| accum.min(val))
}

pub fn point_line_euclidean_distance<C, T>(p: C, l: Line<T>) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    line_segment_distance(p.into(), l.start, l.end)
}

pub fn point_contains_point<T>(p1: Point<T>, p2: Point<T>) -> bool
where
    T: CoordFloat,
{
    let distance = line_euclidean_length(Line::new(p1, p2)).to_f32().unwrap();
    approx::relative_eq!(distance, 0.0)
}

pub fn line_string_contains_point<T>(line_string: &LineString<T>, point: Point<T>) -> bool
where
    T: CoordFloat,
{
    // LineString without points
    if line_string.0.is_empty() {
        return false;
    }
    // LineString with one point equal p
    if line_string.0.len() == 1 {
        return point_contains_point(Point::from(line_string[0]), point);
    }
    // check if point is a vertex
    if line_string.0.contains(&point.0) {
        return true;
    }
    for line in line_string.lines() {
        // This is a duplicate of the line-contains-point logic in the "intersects" module
        let tx = if line.dx() == T::zero() {
            None
        } else {
            Some((point.x() - line.start.x) / line.dx())
        };
        let ty = if line.dy() == T::zero() {
            None
        } else {
            Some((point.y() - line.start.y) / line.dy())
        };
        let contains = match (tx, ty) {
            (None, None) => {
                // Degenerate line
                point.0 == line.start
            }
            (Some(t), None) => {
                // Horizontal line
                point.y() == line.start.y && T::zero() <= t && t <= T::one()
            }
            (None, Some(t)) => {
                // Vertical line
                point.x() == line.start.x && T::zero() <= t && t <= T::one()
            }
            (Some(t_x), Some(t_y)) => {
                // All other lines
                (t_x - t_y).abs() <= T::epsilon() && T::zero() <= t_x && t_x <= T::one()
            }
        };
        if contains {
            return true;
        }
    }
    false
}

pub fn rect_contains_coord<T>(rect: &Rect<T>, coord: &Coord<T>) -> bool
where
    T: CoordNum,
{
    coord.x > rect.min().x
        && coord.x < rect.max().x
        && coord.y > rect.min().y
        && coord.y < rect.max().y
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::assert_relative_eq;

    // https://github.com/georust/geo/issues/1610
    #[test]
    fn line_segment_distance_at_large_coordinates() {
        for y in [-1e100f64, -1e200, -8.988465674311469e307] {
            let start = coord! { x: 0.0, y: y };
            let end = coord! { x: 3.0, y: -0.0 };
            // The end of the segment is the nearest point to the origin.
            assert_relative_eq!(
                line_segment_distance(coord! { x: 0.0, y: 0.0 }, start, end),
                3.0
            );
        }
    }

    // https://github.com/georust/geo/issues/1604
    #[test]
    fn line_segment_distance_at_small_coordinates() {
        let start = coord! { x: 0.0, y: 0.0 };
        let end = coord! { x: 0.0, y: 1e-162 };
        assert_relative_eq!(line_segment_distance(start, start, end), 0.0);
        assert_relative_eq!(
            line_segment_distance(coord! { x: 1e-162, y: 0.0 }, start, end),
            1e-162
        );
    }

    // The projection lands inside a segment whose squared length overflows.
    #[test]
    fn line_segment_distance_perpendicular_at_large_coordinates() {
        let start = coord! { x: 0.0, y: 0.0 };
        let end = coord! { x: 0.0, y: 1e200 };
        assert_relative_eq!(
            line_segment_distance(coord! { x: 3.0, y: 5e199 }, start, end),
            3.0
        );
    }

    // A distant point and a short segment need independent scales.
    #[test]
    fn line_segment_distance_with_a_distant_point() {
        let start = coord! { x: 0.0, y: 0.0 };
        let end = coord! { x: 1.0, y: 0.0 };
        assert_relative_eq!(
            line_segment_distance(coord! { x: 0.0, y: 1e300 }, start, end),
            1e300
        );
        assert_relative_eq!(
            line_segment_distance(coord! { x: -1e300, y: 0.0 }, start, end),
            1e300
        );
    }

    #[test]
    fn line_segment_distance_squared_at_large_coordinates() {
        let start = coord! { x: 0.0, y: -1e200 };
        let end = coord! { x: 3.0, y: -0.0 };
        assert_relative_eq!(
            line_segment_distance_squared(coord! { x: 0.0, y: 0.0 }, start, end),
            9.0
        );
    }

    // The difference of the x coordinates of the segment overflows.
    #[test]
    fn line_segment_distance_where_the_segment_overflows() {
        let point = wkt!(POINT(0.0 0.0));
        let line = wkt!(LINE(9.9792015476736e291 1.0,-1.7976931348623157e308 0.0));
        assert_relative_eq!(line_segment_distance(point.0, line.start, line.end), 1.0);
    }

    // The difference of the x coordinates of the point and the start overflows.
    #[test]
    fn line_segment_distance_where_the_offset_overflows() {
        let point = wkt!(POINT(1.5e308 1.0));
        let line = wkt!(LINE(-1.5e308 0.0,0.0 0.0));
        assert_relative_eq!(
            line_segment_distance(point.0, line.start, line.end),
            1.5e308
        );
    }

    // The point is the midpoint of a segment longer than half the largest finite value.
    #[test]
    fn line_segment_distance_on_a_segment_near_the_largest_finite_value() {
        let point = wkt!(POINT(8.988465674311578e307 9.470887528014527e299));
        let line = wkt!(LINE(0.0 1.8941775056029054e300,1.7976931348623155e308 0.0));
        assert_relative_eq!(line_segment_distance(point.0, line.start, line.end), 0.0);
    }

    // Each extreme is at each position, for lengths that end in every accumulator.
    #[test]
    fn bounding_rect_finds_extremes_at_every_position() {
        let expected = Rect::new(coord! { x: -1.0, y: -4.0 }, coord! { x: 3.0, y: 2.0 });
        for len in 2..=10 {
            for i in 0..len {
                let mut coords = vec![coord! { x: 0.0, y: 0.0 }; len];
                coords[i] = coord! { x: -1.0, y: 2.0 };
                coords[(i + 1) % len] = coord! { x: 3.0, y: -4.0 };
                assert_eq!(get_bounding_rect(&coords), Some(expected), "{coords:?}");
            }
        }
    }

    #[test]
    fn bounding_rect_ignores_nan_after_the_first_coord() {
        let coords = [
            coord! { x: 1.0, y: 1.0 },
            coord! { x: f64::NAN, y: 5.0 },
            coord! { x: 3.0, y: f64::NAN },
        ];
        assert_eq!(
            get_bounding_rect(&coords),
            Some(Rect::new(
                coord! { x: 1.0, y: 1.0 },
                coord! { x: 3.0, y: 5.0 }
            ))
        );
    }

    #[test]
    fn bounding_rect_of_no_coords_is_none() {
        assert_eq!(get_bounding_rect::<_, Coord<f64>, f64>([]), None);
    }
}
