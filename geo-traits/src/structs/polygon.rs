use crate::{
    structs::{Geometry, LineString},
    CoordTrait, Dimensions, GeometryTrait, LineStringTrait, PolygonTrait, RectTrait, TriangleTrait,
};

/// A parsed Polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon<T: Copy = f64> {
    pub(crate) rings: Vec<LineString<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> Polygon<T> {
    /// Create a new Polygon from a sequence of [LineString] and known [Dimensions].
    pub fn new(rings: Vec<LineString<T>>, dim: Dimensions) -> Self {
        Polygon { dim, rings }
    }

    /// Create a new empty polygon.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner rings.
    ///
    /// The first ring is defined to be the exterior ring, and the rest are interior rings.
    pub fn rings(&self) -> &[LineString<T>] {
        &self.rings
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<LineString<T>>, Dimensions) {
        (self.rings, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new polygon from a non-empty sequence of objects implementing [LineStringTrait].
    ///
    /// This will infer the dimension from the first line string, and will not validate that all
    /// line strings have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_rings(
        rings: impl IntoIterator<Item = impl LineStringTrait<T = T>>,
    ) -> Option<Self> {
        let rings = rings
            .into_iter()
            .map(|l| LineString::from_linestring(&l))
            .collect::<Vec<_>>();
        if rings.is_empty() {
            None
        } else {
            let dim = rings[0].dimension();
            Some(Self::new(rings, dim))
        }
    }

    /// Create a new polygon from an object implementing [PolygonTrait].
    pub fn from_polygon(polygon: &impl PolygonTrait<T = T>) -> Self {
        let exterior = polygon.exterior().into_iter();
        let other = polygon.interiors();
        Self::from_rings(exterior.chain(other)).unwrap()
    }

    /// Create a new polygon from an object implementing [TriangleTrait].
    pub fn from_triangle(triangle: &impl TriangleTrait<T = T>) -> Self {
        let ring = super::LineString::from_coords(triangle.coords()).unwrap();
        Self {
            dim: ring.dimension(),
            rings: vec![ring],
        }
    }

    /// Create a new polygon from an object implementing [RectTrait].
    pub fn from_rect(rect: &impl RectTrait<T = T>) -> Self {
        let min = rect.min();
        let max = rect.max();
        // Rect should be 2D, so this just uses X and Y coordinates
        let ring = super::LineString {
            dim: Dimensions::Xy,
            coords: vec![
                super::Coord {
                    x: min.x(),
                    y: min.y(),
                    z: None,
                    m: None,
                },
                super::Coord {
                    x: max.x(),
                    y: min.y(),
                    z: None,
                    m: None,
                },
                super::Coord {
                    x: max.x(),
                    y: max.y(),
                    z: None,
                    m: None,
                },
                super::Coord {
                    x: min.x(),
                    y: max.y(),
                    z: None,
                    m: None,
                },
                super::Coord {
                    x: min.x(),
                    y: min.y(),
                    z: None,
                    m: None,
                },
            ],
        };
        Self {
            rings: vec![ring],
            dim: Dimensions::Xy,
        }
    }
}

impl<T> From<Polygon<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: Polygon<T>) -> Self {
        Geometry::Polygon(value)
    }
}

impl<T: Copy> PolygonTrait for Polygon<T> {
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}

impl<T: Copy> PolygonTrait for &Polygon<T> {
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Coord;
    use crate::PolygonTrait;

    fn square_ring_xy(offset: i32, step: i32) -> LineString<i32> {
        LineString::new(
            vec![
                Coord {
                    x: offset,
                    y: offset,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset + step,
                    y: offset,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset + step,
                    y: offset + step,
                    z: None,
                    m: None,
                },
                Coord {
                    x: offset,
                    y: offset + step,
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
    fn empty_polygon_preserves_dimension() {
        let polygon: Polygon<u8> = Polygon::empty(Dimensions::Xyzm);
        assert_eq!(polygon.dimension(), Dimensions::Xyzm);
        assert!(polygon.rings().is_empty());
    }

    #[test]
    fn from_rings_infers_dimension() {
        let exterior = LineString::new(
            vec![
                Coord {
                    x: 0,
                    y: 0,
                    z: None,
                    m: Some(1),
                },
                Coord {
                    x: 2,
                    y: 0,
                    z: None,
                    m: Some(2),
                },
                Coord {
                    x: 1,
                    y: 2,
                    z: None,
                    m: Some(3),
                },
            ],
            Dimensions::Xym,
        );

        let polygon = Polygon::from_rings(vec![exterior.clone()]).expect("non-empty rings");
        assert_eq!(polygon.dimension(), Dimensions::Xym);
        assert_eq!(polygon.rings(), &[exterior]);
    }

    #[test]
    fn from_rings_returns_none_for_empty_iter() {
        let empty = std::iter::empty::<LineString<i32>>();
        assert!(Polygon::from_rings(empty).is_none());
    }

    #[test]
    fn from_polygon_round_trips_rings() {
        let exterior = square_ring_xy(0, 4);
        let interior = square_ring_xy(1, 2);
        let original = Polygon::new(vec![exterior.clone(), interior.clone()], Dimensions::Xy);

        let converted = Polygon::from_polygon(&original);
        assert_eq!(converted, original);
        assert_eq!(converted.rings(), &[exterior, interior]);
    }

    #[test]
    fn polygon_trait_accessors_work_for_owned_and_borrowed() {
        let exterior = square_ring_xy(0, 2);
        let interior = square_ring_xy(1, 1);
        let polygon = Polygon::new(vec![exterior.clone(), interior.clone()], Dimensions::Xy);

        let ext = polygon.exterior().expect("exterior exists");
        assert_eq!(ext, &exterior);
        assert_eq!(polygon.num_interiors(), 1);
        assert_eq!(polygon.interior(0), Some(&interior));
        assert!(polygon.interior(1).is_none());

        let borrowed = &polygon;
        let borrowed_ext = borrowed.exterior().expect("borrowed exterior exists");
        assert_eq!(borrowed_ext, &exterior);
        assert_eq!(borrowed.num_interiors(), 1);
        assert_eq!(borrowed.interior(0), Some(&interior));
    }
}
