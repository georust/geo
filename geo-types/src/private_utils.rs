// To implement RStar’s traits in the geo-types crates, we need to access to a
// few geospatial algorithms, which are included in this hidden module. This
// hidden module is public so the geo crate can reuse these algorithms to
// prevent duplication. These functions are _not_ meant for public consumption.

use crate::{CoordFloat, CoordNum, Coordinate, Line, LineString, Point, Rect};

pub fn line_string_bounding_rect<T>(line_string: &LineString<T>) -> Option<Rect<T>>
where
    T: CoordNum,
{
    get_bounding_rect(line_string.coords().cloned())
}

pub fn line_bounding_rect<T>(line: Line<T>) -> Rect<T>
where
    T: CoordNum,
{
    Rect::new(line.start, line.end)
}

pub fn get_bounding_rect<I, T>(collection: I) -> Option<Rect<T>>
where
    T: CoordNum,
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
    T: CoordNum,
{
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
    C: Into<Coordinate<T>>,
{
    let point = point.into();
    let start = start.into();
    let end = end.into();

    if start == end {
        return line_euclidean_length(Line::new(point, start));
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let r = ((point.x - start.x) * dx + (point.y - start.y) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new(point, end));
    }
    let s = ((start.y - point.y) * dx - (start.x - point.x) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
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
    C: Into<Coordinate<T>>,
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
        return point_contains_point(Point(line_string[0]), point);
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
