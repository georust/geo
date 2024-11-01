use crate::{coord, CoordFloat, CoordsIter, Polygon, Triangle};

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
    ///             coord! { x: 0., y: 10. }, // NW
    ///             coord! { x: 10., y: 10. }, // NE
    ///             coord! { x: 10., y: 0. }, // SE
    ///         ),
    ///         Triangle(
    ///             coord! { x: 10., y: 0. }, // SE
    ///             coord! { x: 0., y: 0. }, // SW
    ///             coord! { x: 0., y: 10. }, // NW
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
    ///             coord! { x: 0., y: 10. }, // NW
    ///             coord! { x: 10., y: 10. }, // NE
    ///             coord! { x: 10., y: 0. }, // SE
    ///     )),
    ///     triangles_iter.next(),
    /// );
    ///
    /// assert_eq!(
    ///     Some(Triangle(
    ///         coord! { x: 10., y: 0. }, // SE
    ///         coord! { x: 0., y: 0. }, // SW
    ///         coord! { x: 0., y: 10. }, // NW
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
    ///             0., 0., // SW
    ///         ],
    ///         triangle_indices: vec![
    ///             3, 0, 1, // NW-SW-SE
    ///             1, 2, 3, // SE-NE-NW
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
        Some(Triangle(
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

    flat_line_string_coords_2(polygon.exterior(), &mut vertices);

    for interior in polygon.interiors() {
        debug_assert!(interior.0.len() >= 4);
        interior_indexes.push(vertices.len() / 2);
        flat_line_string_coords_2(interior, &mut vertices);
    }

    EarcutrInput {
        vertices,
        interior_indexes,
    }
}

fn flat_line_string_coords_2<T: CoordFloat>(
    line_string: &crate::LineString<T>,
    vertices: &mut Vec<T>,
) {
    for coord in &line_string.0 {
        vertices.push(coord.x);
        vertices.push(coord.y);
    }
}

#[cfg(test)]
mod test {
    use super::TriangulateEarcut;
    use crate::{coord, polygon, Triangle};

    #[test]
    fn test_triangle() {
        let triangle_polygon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 0.),
        ];

        let triangles = triangle_polygon.earcut_triangles();

        assert_eq!(
            &[Triangle(
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 10.0, y: 10.0 },
            ),][..],
            triangles,
        );
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

        let mut triangles = square_polygon.earcut_triangles();
        triangles.sort_by(|t1, t2| t1.1.x.partial_cmp(&t2.2.x).unwrap());

        assert_eq!(
            &[
                Triangle(
                    coord! { x: 10.0, y: 0.0 },
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 0.0, y: 10.0 },
                ),
                Triangle(
                    coord! { x: 0.0, y: 10.0 },
                    coord! { x: 10.0, y: 10.0 },
                    coord! { x: 10.0, y: 0.0 },
                ),
            ][..],
            triangles,
        );
    }
}
