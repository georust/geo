use num_traits::{Float};
use algorithm::intersects::Intersects;
use types::{Point, Line};
use std::fmt::Debug;

pub trait Intersection<Rhs = Self> {
    type OutputGeometry;

    /// Constructs the intersection of two geometries
    fn intersection(&self, rhs: &Rhs) -> Option<Self::OutputGeometry>;
}

#[derive(PartialEq, Debug)]
pub enum LineIntersection<T>
    where T: Float
{
    Point(Point<T>),
    Line(Line<T>)
}

impl<T> Intersection<Line<T>> for Line<T>
    where T: Float + Debug
{
    type OutputGeometry = LineIntersection<T>;
    fn intersection(&self, line: &Line<T>) -> Option<LineIntersection<T>> {
        match (degenerate(&self), degenerate(&line)) {
            (true, true) => {
                if self.start == line.start {
                    Some(LineIntersection::Point(self.start))
                } else {
                    None
                }
            }
            (true, false) => {
                if self.start.intersects(&line) {
                    Some(LineIntersection::Point(self.start))
                } else {
                    None
                }
            }
            (false, true) => {
                if line.start.intersects(&self) {
                    Some(LineIntersection::Point(line.start))
                } else {
                    None
                }
            }
            (false, false) => {
                // Using Cramer's Rule:
                // https://en.wikipedia.org/wiki/Intersection_%28Euclidean_geometry%29#Two_line_segments
                let (x1, y1, x2, y2) = (self.start.x(), self.start.y(),
                                        self.end.x(), self.end.y());
                let (x3, y3, x4, y4) = (line.start.x(), line.start.y(),
                                        line.end.x(), line.end.y());
                // self: (x(s), y(s)) = (x1 + s * a1, y1 + s * a2)
                // line: (x(t), y(t)) = (x4 + t * b1, y4 + t * b2)
                let a1 = x2 - x1;
                let a2 = y2 - y1;
                let b1 = x3 - x4; // == -(x4 - x3)
                let b2 = y3 - y4; // == -(y4 - y3)
                let c1 = x3 - x1;
                let c2 = y3 - y1;
                let d = a1*b2 - a2*b1;
                let u_s = c1*b2 - c2*b1;
                let u_t = a1*c2 - a2*c1;
                if d == T::zero() {
                    // lines are parallel
                    if u_s == T::zero() && u_t == T::zero() {
                        // lines are co-linear
                        let mut endpoints = vec![(self.start, 0), (self.end, 0),
                                                 (line.start, 1), (line.end, 1)];
                        endpoints.sort();
                        let (_, l0) = endpoints[0];
                        let (p1, l1) = endpoints[1];
                        let (p2, _) = endpoints[2];
                        if p1 == p2 {
                            // Line segments overlap at their endpoints
                            Some(LineIntersection::Point(p1))
                        } else if l0 == l1 {
                            // If the first two sorted endpoints belonged to the same line then there is no
                            // overlap
                            None
                        } else {
                            Some(LineIntersection::Line(Line::new(p1, p2)))
                        }
                    } else {
                        // lines are parallel but not co-linear
                        None
                    }
                } else {
                    let s = u_s / d;
                    let t = u_t / d;
                    if (T::zero() <= s) && (s <= T::one()) &&
                       (T::zero() <= t) && (t <= T::one()) {
                        Some(LineIntersection::Point(Point::new(x1 + s * a1,
                                                                y1 + s * a2)))
                    } else {
                        None
                    }
                }
            }
        }
    }
}

fn degenerate<T>(line: &Line<T>) -> bool
    where T: Float
{
    line.start == line.end
}

#[cfg(test)]
mod test {
    use types::{Point, Line};
    use algorithm::intersection::{Intersection, LineIntersection};

    #[test]
    fn test_line_line_intersection(){
        let degenerate_line = Line::new(Point::new(0., 0.), Point::new(0., 0.));
        let l0 = Line::new(Point::new(0., 0.), Point::new(8., 6.));
        let l1 = Line::new(Point::new(0., 6.), Point::new(8., 0.));
        let l2 = Line::new(Point::new(8., 0.), Point::new(8., 10.));
        let l3 = Line::new(Point::new(0., 0.), Point::new(4., 3.));
        let l4 = Line::new(Point::new(-2., -1.5), Point::new(2., 1.5));
        assert_eq!(l0.intersection(&degenerate_line),
                   Some(LineIntersection::Point(Point::new(0., 0.))));
        assert_eq!(l0.intersection(&l0), Some(LineIntersection::Line(l0.clone())));
        assert_eq!(l0.intersection(&l1), Some(LineIntersection::Point(Point::new(4., 3.))));
        assert_eq!(l0.intersection(&l2), Some(LineIntersection::Point(Point::new(8., 6.))));
        assert_eq!(l0.intersection(&l3), Some(LineIntersection::Line(l3.clone())));
        assert_eq!(l0.intersection(&l4),
                   Some(LineIntersection::Line(Line::new(Point::new(0., 0.), Point::new(2., 1.5)))));

        assert_eq!(l1.intersection(&degenerate_line), None);
        assert_eq!(l1.intersection(&l0), Some(LineIntersection::Point(Point::new(4., 3.))));
        assert_eq!(l1.intersection(&l1), Some(LineIntersection::Line(l1.clone())));
        assert_eq!(l1.intersection(&l2), Some(LineIntersection::Point(Point::new(8., 0.))));
        assert_eq!(l1.intersection(&l3), Some(LineIntersection::Point(Point::new(4., 3.))));
        assert_eq!(l1.intersection(&l4), None);

        assert_eq!(l2.intersection(&degenerate_line), None);
        assert_eq!(l2.intersection(&l0), Some(LineIntersection::Point(Point::new(8., 6.))));
        assert_eq!(l2.intersection(&l1), Some(LineIntersection::Point(Point::new(8., 0.))));
        assert_eq!(l2.intersection(&l2), Some(LineIntersection::Line(l2.clone())));
        assert_eq!(l2.intersection(&l3), None);
        assert_eq!(l2.intersection(&l4), None);

        assert_eq!(l3.intersection(&degenerate_line),
                   Some(LineIntersection::Point(Point::new(0., 0.))));
        assert_eq!(l3.intersection(&l0), Some(LineIntersection::Line(l3.clone())));
        assert_eq!(l3.intersection(&l1), Some(LineIntersection::Point(Point::new(4., 3.))));
        assert_eq!(l3.intersection(&l2), None);
        assert_eq!(l3.intersection(&l3), Some(LineIntersection::Line(l3.clone())));
        assert_eq!(l3.intersection(&l4),
                   Some(LineIntersection::Line(Line::new(Point::new(0., 0.), Point::new(2., 1.5)))));
    }
}
