use crate::{CoordFloat, CoordsIter, Polygon, Triangle, coord};

/// Triangulate polygons using an [ear-cutting algorithm](https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf).
///
/// Requires the `"earcutr"` feature, which is enabled by default.
pub trait TriangulateEarcut<T: CoordFloat> {
    /// # Examples
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // SW
    ///     (x: 10., y: 0.), // SE
    ///     (x: 10., y: 10.), // NE
    ///     (x: 0., y: 10.), // NW
    ///     (x: 0., y: 0.), // SW
    /// ];
    ///
    /// let triangles = square_polygon.earcut_triangles();
    ///
    /// assert_eq!(
    ///     vec![
    ///         Triangle(
    ///             coord! { x: 0., y: 0. }, // SW
    ///             coord! { x: 10., y: 0. }, // SE
    ///             coord! { x: 10., y: 10. }, // NE
    ///         ),
    ///         Triangle(
    ///             coord! { x: 10., y: 10. }, // NE
    ///             coord! { x: 0., y: 10. }, // NW
    ///             coord! { x: 0., y: 0. }, // SW
    ///         ),
    ///     ],
    ///     triangles,
    /// );
    /// ```
    fn earcut_triangles(&self) -> Vec<Triangle<T>> {
        self.earcut_triangles_iter().collect()
    }

    /// # Examples
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // SW
    ///     (x: 10., y: 0.), // SE
    ///     (x: 10., y: 10.), // NE
    ///     (x: 0., y: 10.), // NW
    ///     (x: 0., y: 0.), // SW
    /// ];
    ///
    /// let mut triangles_iter = square_polygon.earcut_triangles_iter();
    ///
    /// assert_eq!(
    ///     Some(Triangle(
    ///         coord! { x: 0., y: 0. }, // SW
    ///         coord! { x: 10., y: 0. }, // SE
    ///         coord! { x: 10., y: 10. }, // NE
    ///     )),
    ///     triangles_iter.next(),
    /// );
    ///
    /// assert_eq!(
    ///     Some(Triangle(
    ///         coord! { x: 10., y: 10. }, // NE
    ///         coord! { x: 0., y: 10. }, // NW
    ///         coord! { x: 0., y: 0. }, // SW
    ///     )),
    ///     triangles_iter.next(),
    /// );
    ///
    /// assert!(triangles_iter.next().is_none());
    /// ```
    fn earcut_triangles_iter(&self) -> Iter<T> {
        Iter(self.earcut_triangles_raw())
    }

    /// Return the raw result from the `earcutr` library: a one-dimensional vector of polygon
    /// vertices (in XY order), and the indices of the triangles within the vertices vector. This
    /// method is less ergonomic than the `earcut_triangles` and `earcut_triangles_iter`
    /// methods, but can be helpful when working in graphics contexts that expect flat vectors of
    /// data.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateEarcut};
    /// use geo::triangulate_earcut::RawTriangulation;
    ///
    /// let square_polygon = polygon![
    ///     (x: 0., y: 0.), // SW
    ///     (x: 10., y: 0.), // SE
    ///     (x: 10., y: 10.), // NE
    ///     (x: 0., y: 10.), // NW
    ///     (x: 0., y: 0.), // SW
    /// ];
    ///
    /// let mut triangles_raw = square_polygon.earcut_triangles_raw();
    ///
    /// assert_eq!(
    ///     RawTriangulation {
    ///         vertices: vec![
    ///             0., 0., // SW
    ///             10., 0., // SE
    ///             10., 10., // NE
    ///             0., 10., // NW
    ///         ],
    ///         triangle_indices: vec![
    ///             2, 3, 0, // NE-NW-SW
    ///             0, 1, 2, // SW-SE-NE
    ///         ],
    ///     },
    ///     triangles_raw,
    /// );
    /// ```
    fn earcut_triangles_raw(&self) -> RawTriangulation<T>;
}

impl<T: CoordFloat> TriangulateEarcut<T> for Polygon<T> {
    fn earcut_triangles_raw(&self) -> RawTriangulation<T> {
        let input = polygon_to_earcutr_input(self);
        let triangle_indices =
            earcutr::earcut(&input.vertices, &input.interior_indexes, 2).unwrap();
        RawTriangulation {
            vertices: input.vertices,
            triangle_indices,
        }
    }
}

/// The raw result of triangulating a polygon from `earcutr`.
#[derive(Debug, PartialEq, Clone)]
pub struct RawTriangulation<T: CoordFloat> {
    /// Flattened one-dimensional vector of polygon vertices (in XY order).
    pub vertices: Vec<T>,

    /// Indices of the triangles within the vertices vector.
    pub triangle_indices: Vec<usize>,
}

#[derive(Debug)]
pub struct Iter<T: CoordFloat>(RawTriangulation<T>);

impl<T: CoordFloat> Iterator for Iter<T> {
    type Item = Triangle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let triangle_index_1 = self.0.triangle_indices.pop()?;
        let triangle_index_2 = self.0.triangle_indices.pop()?;
        let triangle_index_3 = self.0.triangle_indices.pop()?;
        Some(Triangle::new(
            self.triangle_index_to_coord(triangle_index_1),
            self.triangle_index_to_coord(triangle_index_2),
            self.triangle_index_to_coord(triangle_index_3),
        ))
    }
}

impl<T: CoordFloat> Iter<T> {
    fn triangle_index_to_coord(&self, triangle_index: usize) -> crate::Coord<T> {
        coord! {
            x: self.0.vertices[triangle_index * 2],
            y: self.0.vertices[triangle_index * 2 + 1],
        }
    }
}

struct EarcutrInput<T: CoordFloat> {
    pub vertices: Vec<T>,
    pub interior_indexes: Vec<usize>,
}

fn polygon_to_earcutr_input<T: CoordFloat>(polygon: &crate::Polygon<T>) -> EarcutrInput<T> {
    let mut vertices = Vec::with_capacity(polygon.coords_count() * 2);
    let mut interior_indexes = Vec::with_capacity(polygon.interiors().len());
    debug_assert!(polygon.exterior().0.len() >= 4);

    flatten_ring(polygon.exterior(), &mut vertices);

    for interior in polygon.interiors() {
        debug_assert!(interior.0.len() >= 4);
        interior_indexes.push(vertices.len() / 2);
        flatten_ring(interior, &mut vertices);
    }

    EarcutrInput {
        vertices,
        interior_indexes,
    }
}

fn flatten_ring<T: CoordFloat>(line_string: &crate::LineString<T>, vertices: &mut Vec<T>) {
    if line_string.0.is_empty() {
        return;
    }
    debug_assert!(line_string.is_closed(), "Only suitable for polygon rings");
    // skip final (redundant) coord for closed line_string to match
    // earcutr's expected input
    for coord in &line_string.0[0..line_string.0.len() - 1] {
        vertices.push(coord.x);
        vertices.push(coord.y);
    }
}

#[cfg(test)]
mod test {
    use super::TriangulateEarcut;
    use crate::{polygon, wkt};

    #[test]
    fn test_triangle() {
        let triangle_polygon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 0.),
        ];

        let triangles = triangle_polygon.earcut_triangles();

        assert_eq!(&[wkt!(TRIANGLE(10.0 0.0,10.0 10.0,0.0 0.0))][..], triangles,);
    }

    #[test]
    fn test_square() {
        let square_polygon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.),
        ];

        let triangles = square_polygon.earcut_triangles();

        assert_eq!(
            &[
                wkt!(TRIANGLE(0.0 0.0,10.0 0.0,10.0 10.0)),
                wkt!(TRIANGLE(10.0 10.0,0.0 10.0,0.0 0.0))
            ][..],
            &triangles,
        );
    }

    #[test]
    fn test_square_raw() {
        let square_polygon = wkt!(POLYGON(
            (0. 0.,10. 0., 10. 10., 0. 10.)
        ));

        let triangles = square_polygon.earcut_triangles_raw();
        assert_eq!(
            triangles.vertices,
            vec![0., 0., 10., 0., 10., 10., 0., 10.] // exterior
        );
        assert_eq!(triangles.triangle_indices, vec![2, 3, 0, 0, 1, 2]);
    }

    #[test]
    fn test_square_with_hole_raw() {
        let poly_with_hole = wkt!(POLYGON(
            (0. 0.,10. 0., 10. 10., 0. 10.),
            (2. 2., 8. 2., 8. 8., 2. 8.)
        ));

        let triangles = poly_with_hole.earcut_triangles_raw();

        assert_eq!(
            triangles.vertices,
            vec![
                0., 0., 10., 0., 10., 10., 0., 10., // exterior
                2., 2., 8., 2., 8., 8., 2., 8., // interior hole
            ]
        );
        assert_eq!(
            triangles.triangle_indices,
            vec![
                3, 0, 7, 4, 7, 0, 2, 3, 7, 5, 4, 0, 2, 7, 6, 5, 0, 1, 1, 2, 6, 6, 5, 1
            ]
        );
    }
}
