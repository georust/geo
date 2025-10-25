use crate::{
    structs::{Geometry, Point},
    Dimensions, MultiPointTrait, PointTrait,
};

/// A parsed MultiPoint.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPoint<T: Copy> {
    pub(crate) points: Vec<Point<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> MultiPoint<T> {
    /// Create a new MultiPoint from a sequence of [Point] and known [Dimensions].
    pub fn new(points: Vec<Point<T>>, dim: Dimensions) -> Self {
        MultiPoint { dim, points }
    }

    /// Create a new empty MultiPoint.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner points.
    pub fn points(&self) -> &[Point<T>] {
        &self.points
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Point<T>>, Dimensions) {
        (self.points, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new MultiPoint from a non-empty sequence of objects implementing [PointTrait].
    ///
    /// This will infer the dimension from the first point, and will not validate that all
    /// points have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_points(points: impl IntoIterator<Item = impl PointTrait<T = T>>) -> Option<Self> {
        let points = points
            .into_iter()
            .map(|p| Point::from_point(p))
            .collect::<Vec<_>>();
        if points.is_empty() {
            None
        } else {
            let dim = points[0].dimension();
            Some(Self::new(points, dim))
        }
    }

    /// Create a new MultiPoint from an objects implementing [MultiPointTrait].
    pub fn from_multipoint(multipoint: impl MultiPointTrait<T = T>) -> Self {
        Self::from_points(multipoint.points()).unwrap()
    }
}

impl<T> From<MultiPoint<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: MultiPoint<T>) -> Self {
        Geometry::MultiPoint(value)
    }
}

impl<T: Copy> MultiPointTrait for MultiPoint<T> {
    type InnerPointType<'a>
        = &'a Point<T>
    where
        Self: 'a;

    fn num_points(&self) -> usize {
        self.points.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
        self.points.get_unchecked(i)
    }
}

impl<T: Copy> MultiPointTrait for &MultiPoint<T> {
    type InnerPointType<'a>
        = &'a Point<T>
    where
        Self: 'a;

    fn num_points(&self) -> usize {
        self.points.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
        self.points.get_unchecked(i)
    }
}
