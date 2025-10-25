use crate::{
    structs::{Geometry, Point},
    CoordTrait, Dimensions, MultiPointTrait, PointTrait,
};

/// A parsed MultiPoint.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPoint<T: Copy = f64> {
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
            .map(|p| Point::from_point(&p))
            .collect::<Vec<_>>();
        if points.is_empty() {
            None
        } else {
            let dim = points[0].dimension();
            Some(Self::new(points, dim))
        }
    }

    /// Create a new MultiPoint from a non-empty sequence of objects implementing [CoordTrait].
    pub fn from_coords(coords: impl IntoIterator<Item = impl CoordTrait<T = T>>) -> Option<Self> {
        let points = coords
            .into_iter()
            .map(|c| Point::from_coord(c))
            .collect::<Vec<_>>();
        if points.is_empty() {
            None
        } else {
            let dim = points[0].dimension();
            Some(Self::new(points, dim))
        }
    }

    /// Create a new MultiPoint from an objects implementing [MultiPointTrait].
    pub fn from_multipoint(multipoint: &impl MultiPointTrait<T = T>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Coord;
    use crate::MultiPointTrait;

    #[test]
    fn empty_multipoint_preserves_dimension() {
        let mp: MultiPoint<i16> = MultiPoint::empty(Dimensions::Xym);
        assert_eq!(mp.dimension(), Dimensions::Xym);
        assert!(mp.points().is_empty());
    }

    #[test]
    fn from_points_infers_dimension() {
        let points = vec![
            Point::new(
                Some(Coord {
                    x: 1,
                    y: 2,
                    z: Some(3),
                    m: None,
                }),
                Dimensions::Xyz,
            ),
            Point::new(
                Some(Coord {
                    x: 4,
                    y: 5,
                    z: Some(6),
                    m: None,
                }),
                Dimensions::Xyz,
            ),
        ];

        let mp = MultiPoint::from_points(points.clone()).expect("points are non-empty");
        assert_eq!(mp.dimension(), Dimensions::Xyz);
        assert_eq!(mp.points(), points.as_slice());
    }

    #[test]
    fn from_points_returns_none_for_empty_iter() {
        let empty = std::iter::empty::<Point<i32>>();
        assert!(MultiPoint::from_points(empty).is_none());
    }

    #[test]
    fn from_multipoint_copies_source() {
        let points = vec![
            Point::new(
                Some(Coord {
                    x: 10,
                    y: 11,
                    z: None,
                    m: Some(1),
                }),
                Dimensions::Xym,
            ),
            Point::new(
                Some(Coord {
                    x: 12,
                    y: 13,
                    z: None,
                    m: Some(2),
                }),
                Dimensions::Xym,
            ),
        ];
        let original = MultiPoint::new(points.clone(), Dimensions::Xym);
        let converted = MultiPoint::from_multipoint(&original);

        assert_eq!(converted.dimension(), original.dimension());
        assert_eq!(converted.points(), original.points());
    }

    #[test]
    fn multipoint_trait_point_access() {
        let mp = MultiPoint::new(
            vec![
                Point::new(
                    Some(Coord {
                        x: 7,
                        y: 8,
                        z: None,
                        m: None,
                    }),
                    Dimensions::Xy,
                ),
                Point::new(
                    Some(Coord {
                        x: 9,
                        y: 10,
                        z: None,
                        m: None,
                    }),
                    Dimensions::Xy,
                ),
            ],
            Dimensions::Xy,
        );

        let mp_ref = &mp;
        let first = mp_ref.point(0).expect("first point exists");
        assert_eq!(first, &mp.points()[0]);
        assert!(mp_ref.point(5).is_none());
    }
}
