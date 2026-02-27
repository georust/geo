use crate::{CoordFloat, CoordsIter, Polygon, Triangle, coord};
use earcut::Earcut;

/// Triangulate polygons using an [ear-cutting algorithm](https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf).
///
/// Requires the `"earcut"` feature, which is enabled by default.
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
        let mut earcut = Earcut::new();

        let input = EarcutInput::from(self);
        // PERF: expose a way to pass in a re-usable output vector for those who are triangulating
        // in a hot loop
        let mut triangle_indices = vec![];
        earcut.earcut(
            input.vertices.clone(), // PERF: iterate `vertices` lazily rather than create a Vec
            &input.interior_indexes,
            &mut triangle_indices,
        );

        RawTriangulation {
            // PERF: As mentioned above, if we don't manifest a concrete vertices Vec,
            // we'll need to return some kind of "getter" that translates triangle_indices back to polygon coords, accounting
            // for the lack of an explicit "closing" coordinate in the triangle_indices
            //
            // We'll need to measure if all that indirection is cheaper than just manifesting the Vec.
            vertices: input.vertices,
            triangle_indices,
        }
    }
}

/// The raw result of triangulating a polygon from `earcutr`.
#[derive(Debug, PartialEq, Clone)]
pub struct RawTriangulation<T: CoordFloat> {
    /// Flattened vector of polygon vertices (in XY order).
    pub vertices: Vec<[T; 2]>,

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
            x: self.0.vertices[triangle_index][0],
            y: self.0.vertices[triangle_index][1],
        }
    }
}

struct EarcutInput<T: CoordFloat> {
    pub vertices: Vec<[T; 2]>,
    pub interior_indexes: Vec<usize>,
}

impl<'a, T: CoordFloat> From<&'a Polygon<T>> for EarcutInput<T> {
    fn from(polygon: &'a Polygon<T>) -> EarcutInput<T> {
        let mut vertices = Vec::with_capacity(polygon.coords_count() - 1);
        let mut interior_indexes = Vec::with_capacity(polygon.interiors().len());
        debug_assert!(polygon.exterior().0.len() >= 4);

        flatten_ring(polygon.exterior(), &mut vertices);

        for interior in polygon.interiors() {
            debug_assert!(interior.0.len() >= 4);
            interior_indexes.push(vertices.len());
            flatten_ring(interior, &mut vertices);
        }

        Self {
            vertices,
            interior_indexes,
        }
    }
}

fn flatten_ring<T: CoordFloat>(line_string: &crate::LineString<T>, vertices: &mut Vec<[T; 2]>) {
    if line_string.0.is_empty() {
        return;
    }
    debug_assert!(line_string.is_closed(), "Only suitable for polygon rings");
    // skip final (redundant) coord for closed line_string to match
    // earcutr's expected input
    for coord in &line_string.0[0..line_string.0.len() - 1] {
        vertices.push([coord.x, coord.y]);
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

        let mut triangles = square_polygon.earcut_triangles();
        triangles.sort_by(|t1, t2| t1.1.x.partial_cmp(&t2.2.x).unwrap());

        assert_eq!(
            &[
                wkt!(TRIANGLE(10.0 10.0,0.0 10.0,0.0 0.0)),
                wkt!(TRIANGLE(0.0 0.0,10.0 0.0,10.0 10.0))
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
            vec![[0., 0.], [10., 0.], [10., 10.], [0., 10.]] // exterior
        );
        assert_eq!(triangles.triangle_indices, vec![2, 3, 0, 0, 1, 2]);
    }

    #[test]
    fn test_square_with_hole_raw() {
        let poly_with_hole = wkt!(POLYGON(
            (0. 0.,10. 0., 10. 10., 0. 10.,0. 0.),
            (2. 2., 8. 2., 8. 8., 2. 8.,2. 2.)
        ));

        let triangles = poly_with_hole.earcut_triangles_raw();

        assert_eq!(
            triangles.vertices,
            vec![
                // exterior
                [0., 0.],
                [10., 0.],
                [10., 10.],
                [0., 10.],
                // interior hole
                [2., 2.],
                [8., 2.],
                [8., 8.],
                [2., 8.],
            ]
        );

        // manually inspected that the output was a reasonable triangulation.
        // let output = poly_with_hole.earcut_triangles();
        // let mp = crate::MultiPolygon::new(output.iter().map(|tri| tri.to_polygon()).collect());
        // dbg!(mp);
        assert_eq!(
            triangles.triangle_indices,
            vec![
                0, 4, 7, 5, 4, 0, 3, 0, 7, 5, 0, 1, 2, 3, 7, 6, 5, 1, 2, 7, 6, 6, 1, 2
            ]
        );
    }
}
