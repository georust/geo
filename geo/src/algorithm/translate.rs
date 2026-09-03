use crate::{AffineOps, AffineTransform, CoordNum};

pub trait Translate<T: CoordNum> {
    /// Translate a Geometry along its axes by the given offsets
    ///
    /// ## Performance
    ///
    /// If you will be performing multiple transformations, like [`Scale`](crate::Scale),
    /// [`Skew`](crate::Skew), [`Translate`], or [`Rotate`](crate::Rotate), it is more
    /// efficient to compose the transformations and apply them as a single operation using the
    /// [`AffineOps`] trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Translate;
    /// use geo::line_string;
    ///
    /// let ls = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 5.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let translated = ls.translate(1.5, 3.5);
    ///
    /// assert_eq!(translated, line_string![
    ///     (x: 1.5, y: 3.5),
    ///     (x: 6.5, y: 8.5),
    ///     (x: 11.5, y: 13.5),
    /// ]);
    /// ```
    #[must_use]
    fn translate(&self, x_offset: T, y_offset: T) -> Self;

    /// Translate a Geometry along its axes, but in place.
    fn translate_mut(&mut self, x_offset: T, y_offset: T);
}

impl<T, G> Translate<T> for G
where
    T: CoordNum,
    G: AffineOps<T>,
{
    fn translate(&self, x_offset: T, y_offset: T) -> Self {
        let transform = AffineTransform::translate(x_offset, y_offset);
        self.affine_transform(&transform)
    }

    fn translate_mut(&mut self, x_offset: T, y_offset: T) {
        let transform = AffineTransform::translate(x_offset, y_offset);
        self.affine_transform_mut(&transform)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Coord, LineString, Polygon, line_string, point, polygon};

    #[test]
    fn test_translate_point() {
        let p = point!(x: 1.0, y: 5.0);
        let translated = p.translate(30.0, 20.0);
        assert_eq!(translated, point!(x: 31.0, y: 25.0));
    }
    #[test]
    fn test_translate_point_in_place() {
        let mut p = point!(x: 1.0, y: 5.0);
        p.translate_mut(30.0, 20.0);
        assert_eq!(p, point!(x: 31.0, y: 25.0));
    }
    #[test]
    fn test_translate_linestring() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 1.0),
            (x: 10.0, y: 0.0),
        ];
        let translated = linestring.translate(17.0, 18.0);
        assert_eq!(
            translated,
            line_string![
                (x: 17.0, y: 18.0),
                (x: 22.0, y: 19.0),
                (x: 27., y: 18.),
            ]
        );
    }
    #[test]
    fn test_translate_polygon() {
        let poly1 = polygon![
            (x: 5., y: 1.),
            (x: 4., y: 2.),
            (x: 4., y: 3.),
            (x: 5., y: 4.),
            (x: 6., y: 4.),
            (x: 7., y: 3.),
            (x: 7., y: 2.),
            (x: 6., y: 1.),
            (x: 5., y: 1.),
        ];
        let translated = poly1.translate(17.0, 18.0);
        let correct = polygon![
            (x: 22.0, y: 19.0),
            (x: 21.0, y: 20.0),
            (x: 21.0, y: 21.0),
            (x: 22.0, y: 22.0),
            (x: 23.0, y: 22.0),
            (x: 24.0, y: 21.0),
            (x: 24.0, y: 20.0),
            (x: 23.0, y: 19.0),
            (x: 22.0, y: 19.0),
        ];
        // results agree with Shapely / GEOS
        assert_eq!(translated, correct);
    }
    #[test]
    fn test_rotate_polygon_holes() {
        let ls1 = LineString::from(vec![
            (5.0, 1.0),
            (4.0, 2.0),
            (4.0, 3.0),
            (5.0, 4.0),
            (6.0, 4.0),
            (7.0, 3.0),
            (7.0, 2.0),
            (6.0, 1.0),
            (5.0, 1.0),
        ]);

        let ls2 = LineString::from(vec![(5.0, 1.3), (5.5, 2.0), (6.0, 1.3), (5.0, 1.3)]);

        let poly1 = Polygon::new(ls1, vec![ls2]);
        let rotated = poly1.translate(17.0, 18.0);
        let correct_outside = vec![
            Coord::from((22.0, 19.0)),
            Coord::from((21.0, 20.0)),
            Coord::from((21.0, 21.0)),
            Coord::from((22.0, 22.0)),
            Coord::from((23.0, 22.0)),
            Coord::from((24.0, 21.0)),
            Coord::from((24.0, 20.0)),
            Coord::from((23.0, 19.0)),
            Coord::from((22.0, 19.0)),
        ];
        let correct_inside = vec![
            Coord::from((22.0, 19.3)),
            Coord::from((22.5, 20.0)),
            Coord::from((23.0, 19.3)),
            Coord::from((22.0, 19.3)),
        ];
        assert_eq!(rotated.exterior().0, correct_outside);
        assert_eq!(rotated.interiors()[0].0, correct_inside);
    }
}

#[cfg(test)]
mod hegel_props {
    use super::Translate;
    use crate::utils::hegel_gens::{coords, geometries, star_polygons};
    use crate::{AffineOps, AffineTransform, Coord, LineString};
    use hegel::generators::{self, Generator, PrintableGenerator};

    /// Line strings with integer coordinates, where translation is exact.
    fn integer_line_strings() -> impl PrintableGenerator<LineString<i32>> {
        generators::vecs(
            generators::tuples!(generators::integers::<i16>(), generators::integers::<i16>())
                .map(|(x, y)| Coord {
                    x: x as i32,
                    y: y as i32,
                })
                .print_as_debug(),
        )
        .map(LineString::new)
        .print_as_debug()
    }

    // `Translate` is implemented for any `CoordNum`, so on integers the offsets
    // cancel exactly and translating back must restore the input.
    #[hegel::test]
    fn translating_integer_coords_and_back_restores_them(tc: hegel::TestCase) {
        let line_string = tc.draw(integer_line_strings());
        let (dx, dy) = tc.draw(generators::tuples!(
            generators::integers::<i16>(),
            generators::integers::<i16>()
        ));
        assert_eq!(
            line_string
                .translate(dx as i32, dy as i32)
                .translate(-(dx as i32), -(dy as i32)),
            line_string
        );
    }

    // "Translate a Geometry along its axes by the given offsets", so successive
    // translations add up.
    #[hegel::test]
    fn successive_translations_add_their_offsets(tc: hegel::TestCase) {
        let line_string = tc.draw(integer_line_strings());
        let a = tc.draw(generators::integers::<i16>()) as i32;
        let b = tc.draw(generators::integers::<i16>()) as i32;
        assert_eq!(
            line_string.translate(a, 0).translate(b, 0),
            line_string.translate(a + b, 0)
        );
    }

    #[hegel::test]
    fn translate_matches_the_affine_translation(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let offset = tc.draw(coords(1e6));
        assert_eq!(
            polygon.translate(offset.x, offset.y),
            polygon.affine_transform(&AffineTransform::translate(offset.x, offset.y))
        );
    }

    // "Apply `transform` immutably, outputting a new geometry" versus "Apply
    // `transform` to mutate `self`".
    #[hegel::test]
    fn translate_mut_matches_translate(tc: hegel::TestCase) {
        let geometry = tc.draw(geometries(1e6));
        let offset = tc.draw(coords(1e6));
        let mut in_place = geometry.clone();
        in_place.translate_mut(offset.x, offset.y);
        assert_eq!(geometry.translate(offset.x, offset.y), in_place);
    }
}
