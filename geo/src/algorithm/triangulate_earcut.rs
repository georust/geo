use crate::{CoordFloat, Polygon, Triangle, coord};
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

    /// Return the raw result from the `earcut` library: a one-dimensional vector of polygon
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
    ///             [0., 0.], // SW
    ///             [10., 0.], // SE
    ///             [10., 10.], // NE
    ///             [0., 10.], // NW
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
        let mut vertices = Vec::new();
        let mut interior_indexes = Vec::new();
        flatten_ring(self.exterior(), &mut vertices);
        for interior in self.interiors() {
            interior_indexes.push(vertices.len());
            flatten_ring(interior, &mut vertices);
        }

        let mut triangle_indices = Vec::new();
        Earcut::new().earcut(
            vertices.iter().copied(),
            &interior_indexes,
            &mut triangle_indices,
        );

        RawTriangulation {
            vertices,
            triangle_indices,
        }
    }
}

/// Reusable triangulator that retains internal buffers across calls,
/// avoiding per-call allocations when triangulating many polygons.
///
/// Methods on the [`TriangulateEarcut`] trait construct a fresh `earcut`
/// instance, vertex buffer, and output buffer on every call. When
/// triangulating many polygons in a hot loop, prefer reusing a single
/// `Earcutter` to amortize those allocations.
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::triangulate_earcut::Earcutter;
///
/// let polygons = vec![
///     polygon![(x: 0., y: 0.), (x: 10., y: 0.), (x: 10., y: 10.), (x: 0., y: 10.)],
///     polygon![(x: 1., y: 1.), (x: 5., y: 1.), (x: 5., y: 5.), (x: 1., y: 5.)],
/// ];
///
/// let mut earcutter = Earcutter::new();
/// for polygon in &polygons {
///     let triangulation = earcutter.triangulate(polygon);
///     for tri in triangulation.triangle_indices.chunks_exact(3) {
///         let v = |i: u32| triangulation.vertices[i as usize];
///         let _ = (v(tri[0]), v(tri[1]), v(tri[2]));
///     }
/// }
/// ```
pub struct Earcutter<T: CoordFloat> {
    earcut: Earcut<T>,
    vertices: Vec<[T; 2]>,
    interior_indexes: Vec<u32>,
    triangle_indices: Vec<u32>,
}

impl<T: CoordFloat> Default for Earcutter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CoordFloat> Earcutter<T> {
    pub fn new() -> Self {
        Self {
            earcut: Earcut::new(),
            vertices: Vec::new(),
            interior_indexes: Vec::new(),
            triangle_indices: Vec::new(),
        }
    }

    /// Triangulate `polygon`, reusing the internal buffers. The returned
    /// [`EarcutTriangulation`] borrows from `self`, so each call must finish
    /// using the previous result before the next call.
    pub fn triangulate(&mut self, polygon: &Polygon<T>) -> EarcutTriangulation<'_, T> {
        self.vertices.clear();
        self.interior_indexes.clear();

        flatten_ring(polygon.exterior(), &mut self.vertices);
        for interior in polygon.interiors() {
            self.interior_indexes.push(self.vertices.len() as u32);
            flatten_ring(interior, &mut self.vertices);
        }

        // `Earcut::earcut` clears `triangle_indices` internally before writing.
        self.earcut.earcut(
            self.vertices.iter().copied(),
            &self.interior_indexes,
            &mut self.triangle_indices,
        );

        EarcutTriangulation {
            vertices: &self.vertices,
            triangle_indices: &self.triangle_indices,
        }
    }
}

/// Borrowed view of a triangulation produced by [`Earcutter::triangulate`].
#[derive(Debug)]
pub struct EarcutTriangulation<'a, T: CoordFloat> {
    /// Flattened polygon vertices (XY pairs).
    pub vertices: &'a [[T; 2]],
    /// Indices into `vertices`, in groups of three per triangle.
    pub triangle_indices: &'a [u32],
}

/// The raw result of triangulating a polygon from `earcut`.
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
    use super::{Earcutter, TriangulateEarcut};
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

    #[test]
    fn test_earcutter_square() {
        let square_polygon = wkt!(POLYGON(
            (0. 0.,10. 0., 10. 10., 0. 10.)
        ));

        let mut earcutter = Earcutter::new();
        let triangulation = earcutter.triangulate(&square_polygon);

        assert_eq!(
            triangulation.vertices,
            &[[0., 0.], [10., 0.], [10., 10.], [0., 10.]][..]
        );
        assert_eq!(triangulation.triangle_indices, &[2u32, 3, 0, 0, 1, 2][..]);
    }

    #[test]
    fn test_earcutter_reuse_shrinks_buffers() {
        let square = wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10.)));
        let triangle = wkt!(POLYGON((0. 0., 10. 0., 10. 10.)));

        let mut earcutter = Earcutter::new();

        // Square: 4 vertices, 6 triangle indices.
        let result = earcutter.triangulate(&square);
        assert_eq!(result.vertices.len(), 4);
        assert_eq!(result.triangle_indices.len(), 6);

        // Triangle: 3 vertices, 3 triangle indices. Buffers must shrink, not append.
        let result = earcutter.triangulate(&triangle);
        assert_eq!(result.vertices.len(), 3);
        assert_eq!(result.triangle_indices.len(), 3);
    }
}
