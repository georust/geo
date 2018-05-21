// FIXME: everything in this file is copy/paste from the 'geo' crate. ideally we
//        wouldn't have this duplication

use num_traits::{Float, ToPrimitive};
use {CoordinateType, Line, Point};

pub trait EuclideanDistance<T, Rhs = Self> {
    fn euclidean_distance(&self, rhs: &Rhs) -> T;
}

fn line_segment_distance<T>(point: &Point<T>, start: &Point<T>, end: &Point<T>) -> T
where
    T: Float + ToPrimitive,
{
    if start == end {
        return point.euclidean_distance(start);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r =
        ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return point.euclidean_distance(start);
    }
    if r >= T::one() {
        return point.euclidean_distance(end);
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * (dx * dx + dy * dy).sqrt()
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

impl<T> EuclideanDistance<T, Point<T>> for Line<T>
where
    T: Float,
{
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        line_segment_distance(point, &self.start, &self.end)
    }
}

pub trait BoundingBox<T: CoordinateType> {
    type Output;

    fn bbox(&self) -> Self::Output;
}

impl<T> BoundingBox<T> for Line<T>
where
    T: CoordinateType,
{
    type Output = Bbox<T>;

    fn bbox(&self) -> Self::Output {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x() <= b.x() {
            (a.x(), b.x())
        } else {
            (b.x(), a.x())
        };
        let (ymin, ymax) = if a.y() <= b.y() {
            (a.y(), b.y())
        } else {
            (b.y(), a.y())
        };
        Bbox {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Bbox<T>
where
    T: CoordinateType,
{
    pub xmin: T,
    pub xmax: T,
    pub ymin: T,
    pub ymax: T,
}
