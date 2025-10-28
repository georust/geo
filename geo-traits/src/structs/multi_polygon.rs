use crate::{
    structs::{Geometry, Polygon},
    Dimensions, MultiPolygonTrait, PolygonTrait,
};

/// A parsed MultiPolygon.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPolygon<T: Copy = f64> {
    pub(crate) polygons: Vec<Polygon<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> MultiPolygon<T> {
    /// Create a new MultiPolygon from a sequence of [Polygon] and known [Dimensions].
    pub fn new(polygons: Vec<Polygon<T>>, dim: Dimensions) -> Self {
        MultiPolygon { dim, polygons }
    }

    /// Create a new empty MultiPolygon.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner polygons.
    pub fn polygons(&self) -> &[Polygon<T>] {
        &self.polygons
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Polygon<T>>, Dimensions) {
        (self.polygons, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new MultiPolygon from a non-empty sequence of objects implementing [PolygonTrait].
    ///
    /// This will infer the dimension from the first polygon, and will not validate that all
    /// polygons have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty; while an empty MULTIPOLYGON is valid, the
    /// dimension cannot be inferred.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_polygons(
        polygons: impl IntoIterator<Item = impl PolygonTrait<T = T>>,
    ) -> Option<Self> {
        let polygons = polygons
            .into_iter()
            .map(|p| Polygon::from_polygon(&p))
            .collect::<Vec<_>>();
        if polygons.is_empty() {
            None
        } else {
            let dim = polygons[0].dimension();
            Some(Self::new(polygons, dim))
        }
    }

    pub(crate) fn from_polygons_with_dim(
        polygons: impl IntoIterator<Item = impl PolygonTrait<T = T>>,
        dim: Dimensions,
    ) -> Self {
        match Self::from_polygons(polygons) {
            Some(multipolygon) => multipolygon,
            None => Self {
                polygons: Vec::new(),
                dim,
            },
        }
    }

    /// Create a new MultiPolygon from an objects implementing [MultiPolygonTrait].
    pub fn from_multi_polygon(multi_polygon: &impl MultiPolygonTrait<T = T>) -> Self {
        Self::from_polygons_with_dim(multi_polygon.polygons(), multi_polygon.dim())
    }
}

impl<T> From<MultiPolygon<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: MultiPolygon<T>) -> Self {
        Geometry::MultiPolygon(value)
    }
}

impl<T: Copy> MultiPolygonTrait for MultiPolygon<T> {
    type InnerPolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}

impl<T: Copy> MultiPolygonTrait for &MultiPolygon<T> {
    type InnerPolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::{Coord, LineString};
    use crate::MultiPolygonTrait;

    fn square_ring_xy(offset: i32, size: i32) -> LineString<i32> {
        LineString::new(
            vec![
                Coord {
                    x: offset,
                    y: offset,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset + size,
                    y: offset,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset + size,
                    y: offset + size,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset,
                    y: offset + size,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset,
                    y: offset,
                    z: None,
                    m: None,
                },
            ],
            Dimensions::Xy,
        )
    }

    #[test]
    fn empty_multipolygon_preserves_dimension() {
        let mp: MultiPolygon<u8> = MultiPolygon::empty(Dimensions::Xyzm);
        assert_eq!(mp.dimension(), Dimensions::Xyzm);
        assert!(mp.polygons().is_empty());
    }

    #[test]
    fn from_polygons_infers_dimension() {
        let exterior = LineString::new(
            vec![
                Coord {
                    x: 0,
                    y: 0,
                    z: None,
                    m: Some(1),
                },
                Coord {
                    x: 3,
                    y: 0,
                    z: None,
                    m: Some(2),
                },
                Coord {
                    x: 3,
                    y: 3,
                    z: None,
                    m: Some(3),
                },
                Coord {
                    x: 0,
                    y: 0,
                    z: None,
                    m: Some(4),
                },
            ],
            Dimensions::Xym,
        );
        let polygon = Polygon::new(vec![exterior.clone()], Dimensions::Xym);
        let mp =
            MultiPolygon::from_polygons(vec![polygon.clone()]).expect("polygons are non-empty");

        assert_eq!(mp.dimension(), Dimensions::Xym);
        assert_eq!(mp.polygons(), &[polygon]);
    }

    #[test]
    fn from_polygons_returns_none_for_empty_iter() {
        let empty = std::iter::empty::<Polygon<i32>>();
        assert!(MultiPolygon::from_polygons(empty).is_none());
    }

    #[test]
    fn from_multipolygon_copies_source() {
        let polygon_a = Polygon::new(vec![square_ring_xy(0, 2)], Dimensions::Xy);
        let polygon_b = Polygon::new(vec![square_ring_xy(3, 2)], Dimensions::Xy);
        let original =
            MultiPolygon::new(vec![polygon_a.clone(), polygon_b.clone()], Dimensions::Xy);

        let converted = MultiPolygon::from_multi_polygon(&original);
        assert_eq!(converted.dimension(), original.dimension());
        assert_eq!(converted.polygons(), original.polygons());
    }

    #[test]
    fn multipolygon_trait_accessors_work() {
        let poly_a = Polygon::new(vec![square_ring_xy(0, 3)], Dimensions::Xy);
        let poly_b = Polygon::new(vec![square_ring_xy(5, 1)], Dimensions::Xy);
        let mp = MultiPolygon::new(vec![poly_a.clone(), poly_b.clone()], Dimensions::Xy);

        assert_eq!(mp.num_polygons(), 2);
        assert_eq!(mp.polygon(0), Some(&poly_a));
        assert!(mp.polygon(3).is_none());

        let borrowed = &mp;
        let second = borrowed.polygon(1).expect("second polygon exists");
        assert_eq!(second, &poly_b);
    }
}
