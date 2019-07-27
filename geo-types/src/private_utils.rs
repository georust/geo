// To implement RStar’s traits in the geo-types crates, we need to access to a
// few geospatial algorithms, which are included in this hidden module. This
// hidden module is public so the geo crate can reuse these algorithms to
// prevent duplication. These functions are _not_ meant for public consumption.

use crate::{Coordinate, CoordinateType, Line, LineString, Point, Rect};
use num_traits::Float;

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

pub fn line_string_bounding_rect<T>(line_string: &LineString<T>) -> Option<Rect<T>>
where
    T: CoordinateType,
{
    get_bounding_rect(line_string.0.iter().cloned())
}

pub fn line_bounding_rect<T>(line: Line<T>) -> Rect<T>
where
    T: CoordinateType,
{
    let a = line.start;
    let b = line.end;
    let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
    let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };

    Rect::new(
        Coordinate { x: xmin, y: ymin },
        Coordinate { x: xmax, y: ymax },
    )
}

pub fn get_bounding_rect<I, T>(collection: I) -> Option<Rect<T>>
where
    T: CoordinateType,
    I: IntoIterator<Item = Coordinate<T>>,
{
    let mut iter = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        for pnt in iter {
            let (px, py) = pnt.x_y();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }

        return Some(Rect::new(
            Coordinate {
                x: xrange.0,
                y: yrange.0,
            },
            Coordinate {
                x: xrange.1,
                y: yrange.1,
            },
        ));
    }
    None
}

fn get_min_max<T>(p: T, min: T, max: T) -> (T, T)
where
    T: CoordinateType,
{
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}

pub fn line_segment_distance<T>(point: Point<T>, start: Point<T>, end: Point<T>) -> T
where
    T: Float,
{
    if start == end {
        return line_euclidean_length(Line::new(point, start));
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r =
        ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new(point, end));
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
}

pub fn line_euclidean_length<T>(line: Line<T>) -> T
where
    T: Float,
{
    line.dx().hypot(line.dy())
}

pub fn point_line_string_euclidean_distance<T>(p: Point<T>, l: &LineString<T>) -> T
where
    T: Float,
{
    // No need to continue if the point is on the LineString, or it's empty
    if line_string_contains_point(l, p) || l.0.is_empty() {
        return T::zero();
    }
    l.lines()
        .map(|line| line_segment_distance(p, line.start_point(), line.end_point()))
        .fold(T::max_value(), |accum, val| accum.min(val))
}

pub fn point_line_euclidean_distance<T>(p: Point<T>, l: Line<T>) -> T
where
    T: Float,
{
    line_segment_distance(p, l.start_point(), l.end_point())
}

pub fn point_contains_point<T>(p1: Point<T>, p2: Point<T>) -> bool
where
    T: Float,
{
    line_euclidean_length(Line::new(p1, p2)).to_f32().unwrap() < COORD_PRECISION
}

pub fn line_string_contains_point<T>(line_string: &LineString<T>, point: Point<T>) -> bool
where
    T: Float,
{
    // LineString without points
    if line_string.0.is_empty() {
        return false;
    }
    // LineString with one point equal p
    if line_string.0.len() == 1 {
        return point_contains_point(Point(line_string.0[0]), point);
    }
    // check if point is a vertex
    if line_string.0.contains(&point.0) {
        return true;
    }
    for line in line_string.lines() {
        if ((line.start.y == line.end.y)
            && (line.start.y == point.y())
            && (point.x() > line.start.x.min(line.end.x))
            && (point.x() < line.start.x.max(line.end.x)))
            || ((line.start.x == line.end.x)
                && (line.start.x == point.x())
                && (point.y() > line.start.y.min(line.end.y))
                && (point.y() < line.start.y.max(line.end.y)))
        {
            return true;
        }
    }
    false
}
