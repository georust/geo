use crate::map_coords::{MapCoords, MapCoordsInPlace};
use crate::{CoordNum, Coordinate};

pub trait Translate<T> {
    /// Translate a Geometry along its axes by the given offsets
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
    fn translate(&self, xoff: T, yoff: T) -> Self
    where
        T: CoordNum;

    /// Translate a Geometry along its axes, but in place.
    fn translate_in_place(&mut self, xoff: T, yoff: T)
    where
        T: CoordNum;

    /// Translate a Geometry along its axes, but in place.
    #[deprecated(since = "0.20.1", note = "renamed to `translate_in_place`")]
    fn translate_inplace(&mut self, xoff: T, yoff: T)
    where
        T: CoordNum;
}

impl<T, G> Translate<T> for G
where
    T: CoordNum,
    G: MapCoords<T, T, Output = G> + MapCoordsInPlace<T>,
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        self.map_coords(|Coordinate { x, y }| Coordinate {
            x: x + xoff,
            y: y + yoff,
        })
    }

    fn translate_in_place(&mut self, xoff: T, yoff: T) {
        self.map_coords_in_place(|Coordinate { x, y }| Coordinate {
            x: x + xoff,
            y: y + yoff,
        })
    }

    fn translate_inplace(&mut self, xoff: T, yoff: T) {
        self.translate_in_place(xoff, yoff)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, point, polygon, Coordinate, LineString, Polygon};

    #[test]
    fn test_translate_point() {
        let p = point!(x: 1.0, y: 5.0);
        let translated = p.translate(30.0, 20.0);
        assert_eq!(translated, point!(x: 31.0, y: 25.0));
    }
    #[test]
    fn test_translate_point_in_place() {
        let mut p = point!(x: 1.0, y: 5.0);
        p.translate_in_place(30.0, 20.0);
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
            Coordinate::from((22.0, 19.0)),
            Coordinate::from((21.0, 20.0)),
            Coordinate::from((21.0, 21.0)),
            Coordinate::from((22.0, 22.0)),
            Coordinate::from((23.0, 22.0)),
            Coordinate::from((24.0, 21.0)),
            Coordinate::from((24.0, 20.0)),
            Coordinate::from((23.0, 19.0)),
            Coordinate::from((22.0, 19.0)),
        ];
        let correct_inside = vec![
            Coordinate::from((22.0, 19.3)),
            Coordinate::from((22.5, 20.0)),
            Coordinate::from((23.0, 19.3)),
            Coordinate::from((22.0, 19.3)),
        ];
        assert_eq!(rotated.exterior().0, correct_outside);
        assert_eq!(rotated.interiors()[0].0, correct_inside);
    }
}
