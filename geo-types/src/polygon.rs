use num_traits::{Float, Signed};
use {CoordinateType, LineString};

/// A representation of an area. Its outer boundary is represented by a [`LineString`](struct.LineString.html) that is both closed and simple
///
/// It has one exterior *ring* or *shell*, and zero or more interior rings, representing holes.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon<T>
where
    T: CoordinateType,
{
    pub exterior: LineString<T>,
    pub interiors: Vec<LineString<T>>,
}

impl<T> Polygon<T>
where
    T: CoordinateType,
{
    /// Creates a new polygon.
    ///
    /// ```
    /// use geo_types::{Point, LineString, Polygon};
    ///
    /// let exterior = LineString(vec![Point::new(0., 0.), Point::new(1., 1.),
    ///                                Point::new(1., 0.), Point::new(0., 0.)]);
    /// let interiors = vec![LineString(vec![Point::new(0.1, 0.1), Point::new(0.9, 0.9),
    ///                                      Point::new(0.9, 0.1), Point::new(0.1, 0.1)])];
    /// let p = Polygon::new(exterior.clone(), interiors.clone());
    /// assert_eq!(p.exterior, exterior);
    /// assert_eq!(p.interiors, interiors);
    /// ```
    pub fn new(exterior: LineString<T>, interiors: Vec<LineString<T>>) -> Polygon<T> {
        Polygon {
            exterior: exterior,
            interiors: interiors,
        }
    }
    /// Wrap-around previous-vertex
    fn previous_vertex(&self, current_vertex: &usize) -> usize
    where
        T: Float,
    {
        (current_vertex + (self.exterior.0.len() - 1) - 1) % (self.exterior.0.len() - 1)
    }
}

// used to check the sign of a vec of floats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ListSign {
    Empty,
    Positive,
    Negative,
    Mixed,
}

impl<T> Polygon<T>
where
    T: Float + Signed,
{
    /// Determine whether a Polygon is convex
    // For each consecutive pair of edges of the polygon (each triplet of points),
    // compute the z-component of the cross product of the vectors defined by the
    // edges pointing towards the points in increasing order.
    // Take the cross product of these vectors
    // The polygon is convex if the z-components of the cross products are either
    // all positive or all negative. Otherwise, the polygon is non-convex.
    // see: http://stackoverflow.com/a/1881201/416626
    pub fn is_convex(&self) -> bool {
        let convex = self
            .exterior
            .0
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let prev_1 = self.previous_vertex(&idx);
                let prev_2 = self.previous_vertex(&prev_1);
                self.exterior.0[prev_2].cross_prod(
                    &self.exterior.0[prev_1],
                    &self.exterior.0[idx]
                )
            })
            // accumulate and check cross-product result signs in a single pass
            // positive implies ccw convexity, negative implies cw convexity
            // anything else implies non-convexity
            .fold(ListSign::Empty, |acc, n| {
                match (acc, n.is_positive()) {
                    (ListSign::Empty, true) | (ListSign::Positive, true) => ListSign::Positive,
                    (ListSign::Empty, false) | (ListSign::Negative, false) => ListSign::Negative,
                    _ => ListSign::Mixed
                }
            });
        convex != ListSign::Mixed
    }
}
