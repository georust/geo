use num_traits::{Float, Signed};
use crate::{CoordinateType, LineString, Point, Rect};

/// A bounded two-dimensional area.
///
/// A `Polygon`’s outer boundary (_exterior ring_) is represented by a
/// [`LineString`]. It may contain zero or more holes (_interior rings_), also
/// represented by `LineString`s.
///
/// The `Polygon` structure guarantees that all exterior and interior rings will
/// be _closed_, such that the first and last `Coordinate` of each ring has
/// the same value.
///
/// # Validity
///
/// Besides the closed `LineString` rings guarantee, the `Polygon` structure
/// does not enforce validity at this time. For example, it is possible to
/// construct a `Polygon` that has:
///
/// - fewer than 3 coordinates per `LineString` ring
/// - interior rings that intersect other interior rings
/// - interior rings that extend beyond the exterior ring
///
/// # `LineString` closing operation
///
/// Some APIs on `Polygon` result in a closing operation on a `LineString`. The
/// operation is as follows:
///
/// If a `LineString`’s first and last `Coordinate` have different values, a
/// new `Coordinate` will be appended to the `LineString` with a value equal to
/// the first `Coordinate`.
///
/// [`LineString`]: struct.LineString.html
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon<T>
where
    T: CoordinateType,
{
    exterior: LineString<T>,
    interiors: Vec<LineString<T>>,
}

impl<T> Polygon<T>
where
    T: CoordinateType,
{
    /// Create a new `Polygon` with the provided exterior `LineString` ring and
    /// interior `LineString` rings.
    ///
    /// Upon calling `new`, the exterior and interior `LineString` rings [will
    /// be closed].
    ///
    /// [will be closed]: #linestring-closing-operation
    ///
    /// # Examples
    ///
    /// Creating a `Polygon` with no interior rings:
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![]);
    /// ```
    ///
    /// Creating a `Polygon` with an interior ring:
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![
    ///     LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])
    /// ]);
    /// ```
    ///
    /// If the first and last `Coordinate`s of the exterior or interior
    /// `LineString`s no longer match, those `LineString`s [will be closed]:
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    /// ]), vec![]);
    ///
    /// assert_eq!(polygon.exterior(), &LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]));
    /// ```
    pub fn new(mut exterior: LineString<T>, mut interiors: Vec<LineString<T>>) -> Polygon<T> {
        exterior.close();
        for interior in &mut interiors {
            interior.close();
        }
        Polygon {
            exterior,
            interiors,
        }
    }

    /// Consume the `Polygon`, returning the exterior `LineString` ring and
    /// a vector of the interior `LineString` rings.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![
    ///     LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])
    /// ]);
    ///
    /// let (exterior, interiors) = polygon.into_inner();
    ///
    /// assert_eq!(exterior, LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]));
    ///
    /// assert_eq!(interiors, vec![LineString::from(vec![
    ///     (0.1, 0.1),
    ///     (0.9, 0.9),
    ///     (0.9, 0.1),
    ///     (0.1, 0.1),
    /// ])]);
    /// ```
    pub fn into_inner(self) -> (LineString<T>, Vec<LineString<T>>) {
        (self.exterior, self.interiors)
    }

    /// Return a reference to the exterior `LineString` ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let exterior = LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]);
    ///
    /// let polygon = Polygon::new(exterior.clone(), vec![]);
    ///
    /// assert_eq!(polygon.exterior(), &exterior);
    /// ```
    pub fn exterior(&self) -> &LineString<T> {
        &self.exterior
    }

    /// Execute the provided closure `f`, which is provided with a mutable
    /// reference to the exterior `LineString` ring.
    ///
    /// After the closure executes, the exterior `LineString` [will be closed].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![]);
    ///
    /// polygon.exterior_mut(|exterior| {
    ///     exterior.0[1] = Coordinate { x: 1., y: 2. };
    /// });
    ///
    /// assert_eq!(polygon.exterior(), &LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 2.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]));
    /// ```
    ///
    /// If the first and last `Coordinate`s of the exterior `LineString` no
    /// longer match, the `LineString` [will be closed]:
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![]);
    ///
    /// polygon.exterior_mut(|exterior| {
    ///     exterior.0[0] = Coordinate { x: 0., y: 1. };
    /// });
    ///
    /// assert_eq!(polygon.exterior(), &LineString::from(vec![
    ///     (0., 1.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    ///     (0., 1.),
    /// ]));
    /// ```
    ///
    /// [will be closed]: #linestring-closing-operation
    pub fn exterior_mut<F>(&mut self, mut f: F)
        where F: FnMut(&mut LineString<T>)
    {
        f(&mut self.exterior);
        self.exterior.close();
    }

    /// Return a slice of the interior `LineString` rings.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let interiors = vec![LineString::from(vec![
    ///     (0.1, 0.1),
    ///     (0.9, 0.9),
    ///     (0.9, 0.1),
    ///     (0.1, 0.1),
    /// ])];
    ///
    /// let polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), interiors.clone());
    ///
    /// assert_eq!(interiors, polygon.interiors());
    /// ```
    pub fn interiors(&self) -> &[LineString<T>] {
        &self.interiors
    }

    /// Execute the provided closure `f`, which is provided with a mutable
    /// reference to the interior `LineString` rings.
    ///
    /// After the closure executes, each of the interior `LineString`s [will be
    /// closed].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![
    ///     LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])
    /// ]);
    ///
    /// polygon.interiors_mut(|interiors| {
    ///     interiors[0].0[1] = Coordinate { x: 0.8, y: 0.8 };
    /// });
    ///
    /// assert_eq!(polygon.interiors(), &[
    ///     LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.8, 0.8),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])
    /// ]);
    /// ```
    ///
    /// If the first and last `Coordinate`s of any interior `LineString` no
    /// longer match, those `LineString`s [will be closed]:
    ///
    /// ```
    /// use geo_types::{Coordinate, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (1., 1.),
    ///     (1., 0.),
    ///     (0., 0.),
    /// ]), vec![
    ///     LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])
    /// ]);
    ///
    /// polygon.interiors_mut(|interiors| {
    ///     interiors[0].0[0] = Coordinate { x: 0.1, y: 0.2 };
    /// });
    ///
    /// assert_eq!(polygon.interiors(), &[
    ///     LineString::from(vec![
    ///         (0.1, 0.2),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///         (0.1, 0.2),
    ///     ])
    /// ]);
    /// ```
    ///
    /// [will be closed]: #linestring-closing-operation
    pub fn interiors_mut<F>(&mut self, mut f: F)
        where F: FnMut(&mut [LineString<T>])
    {
        f(&mut self.interiors);
        for mut interior in &mut self.interiors {
            interior.close();
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
                Point(self.exterior.0[prev_2])
                    .cross_prod(Point(self.exterior.0[prev_1]), Point(self.exterior.0[idx]))
            })
            // accumulate and check cross-product result signs in a single pass
            // positive implies ccw convexity, negative implies cw convexity
            // anything else implies non-convexity
            .fold(ListSign::Empty, |acc, n| match (acc, n.is_positive()) {
                (ListSign::Empty, true) | (ListSign::Positive, true) => ListSign::Positive,
                (ListSign::Empty, false) | (ListSign::Negative, false) => ListSign::Negative,
                _ => ListSign::Mixed,
            });
        convex != ListSign::Mixed
    }
}

impl<T: CoordinateType> From<Rect<T>> for Polygon<T> {
    fn from(r: Rect<T>) -> Polygon<T> {
        Polygon::new(
            vec![
                (r.min.x, r.min.y),
                (r.max.x, r.min.y),
                (r.max.x, r.max.y),
                (r.min.x, r.max.y),
                (r.min.x, r.min.y),
            ]
            .into(),
            Vec::new(),
        )
    }
}
