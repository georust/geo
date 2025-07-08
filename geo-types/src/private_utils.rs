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
    if let Some(pnt) = iter.next() {
        let pnt = pnt.as_ref();
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        for pnt in iter {
            let (px, py) = pnt.as_ref().x_y();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }

        return Some(Rect::new(
            coord! {
                x: xrange.0,
                y: yrange.0,
            },
            coord! {
                x: xrange.1,
                y: yrange.1,
            },
        ));
    }
    None
}

fn get_min_max<T: PartialOrd>(p: T, min: T, max: T) -> (T, T) {
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}

pub fn line_segment_distance<T, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    line_segment_distance_squared(point, start, end).sqrt()
}

pub fn line_segment_distance_squared<T, C>(point: C, start: C, end: C) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    let point = point.into();
    let start = start.into();
    let end = end.into();

    // Degenerate case for line with length 0 - treat as a point
    if start == end {
        return line_euclidean_length_squared(Line::new(point, start));
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let d_squared = dx * dx + dy * dy;

    // Projection of point onto the line segment
    let r = ((point.x - start.x) * dx + (point.y - start.y) * dy) / d_squared;
    // Projection lies beyond start - start point is closest
    if r <= T::zero() {
        return line_euclidean_length_squared(Line::new(point, start));
    }
    // Projection lies beyond end - end point is closest
    if r >= T::one() {
        return line_euclidean_length_squared(Line::new(point, end));
    }
    // Projection lies on midpoint between start-end
    let s = ((start.y - point.y) * dx - (start.x - point.x) * dy) / d_squared;
    s.powi(2) * d_squared
}

pub fn line_euclidean_length<T>(line: Line<T>) -> T
where
    T: CoordFloat,
{
    line_euclidean_length_squared(line).sqrt()
}

pub fn line_euclidean_length_squared<T>(line: Line<T>) -> T
where
    T: CoordFloat,
{
    line.dx().powi(2) + line.dy().powi(2)
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

pub fn point_line_euclidean_distance_squared<C, T>(p: C, l: Line<T>) -> T
where
    T: CoordFloat,
    C: Into<Coord<T>>,
{
    line_segment_distance_squared(p.into(), l.start, l.end)
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
