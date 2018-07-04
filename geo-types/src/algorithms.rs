// FIXME: everything in this file is copy/paste from the 'geo' crate. ideally we
//        wouldn't have this duplication

use num_traits::{Float, ToPrimitive};
use {Coordinate, CoordinateType, Line, LineString, Point, Rect};

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T> Contains<Point<T>> for Point<T>
where
    T: Float + ToPrimitive,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.euclidean_distance(p).to_f32().unwrap() < COORD_PRECISION
    }
}

impl<T> Contains<Point<T>> for LineString<T>
where
    T: Float,
{
    fn contains(&self, p: &Point<T>) -> bool {
        // LineString without points
        if self.0.is_empty() {
            return false;
        }
        // LineString with one point equal p
        if self.0.len() == 1 {
            return Point(self.0[0]).contains(p);
        }
        // check if point is a vertex
        if self.0.contains(&p.0) {
            return true;
        }
        for line in self.lines() {
            if ((line.start.y == line.end.y)
                && (line.start.y == p.y())
                && (p.x() > line.start.x.min(line.end.x))
                && (p.x() < line.start.x.max(line.end.x)))
                || ((line.start.x == line.end.x)
                    && (line.start.x == p.x())
                    && (p.y() > line.start.y.min(line.end.y))
                    && (p.y() < line.start.y.max(line.end.y)))
            {
                return true;
            }
        }
        false
    }
}

pub trait EuclideanDistance<T, Rhs = Self> {
    fn euclidean_distance(&self, rhs: &Rhs) -> T;
}

fn line_segment_distance<T>(point: Point<T>, start: Point<T>, end: Point<T>) -> T
where
    T: Float + ToPrimitive,
{
    if start == end {
        return point.euclidean_distance(&start);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r =
        ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return point.euclidean_distance(&start);
    }
    if r >= T::one() {
        return point.euclidean_distance(&end);
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
}

impl<T> EuclideanDistance<T, Point<T>> for Point<T>
where
    T: Float,
{
    fn euclidean_distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        dx.hypot(dy)
    }
}

impl<T> EuclideanDistance<T, LineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a LineString
    fn euclidean_distance(&self, linestring: &LineString<T>) -> T {
        // No need to continue if the point is on the LineString, or it's empty
        if linestring.contains(self) || linestring.0.is_empty() {
            return T::zero();
        }
        linestring
            .lines()
            .map(|line| line_segment_distance(*self, line.start_point(), line.end_point()))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> EuclideanDistance<T, Point<T>> for LineString<T>
where
    T: Float,
{
    /// Minimum distance from a LineString to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, Point<T>> for Line<T>
where
    T: Float,
{
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        let (start, end) = self.points();
        line_segment_distance(*point, start, end)
    }
}

pub trait BoundingRect<T: CoordinateType> {
    type Output;

    fn bounding_rect(&self) -> Self::Output;
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

fn get_bounding_rect<I, T>(collection: I) -> Option<Rect<T>>
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
        return Some(Rect {
            min: Coordinate {
                x: xrange.0,
                y: yrange.0,
            },
            max: Coordinate {
                x: xrange.1,
                y: yrange.1,
            },
        });
    }
    None
}

impl<T> BoundingRect<T> for Line<T>
where
    T: CoordinateType,
{
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Rect {
            min: Coordinate { x: xmin, y: ymin },
            max: Coordinate { x: xmax, y: ymax },
        }
    }
}

impl<T> BoundingRect<T> for LineString<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the bounding rectangle for a LineString
    ///
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.0.iter().cloned())
    }
}
