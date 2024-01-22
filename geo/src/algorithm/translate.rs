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
    use crate::{line_string, point, polygon, Coord, LineString, Polygon};

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
