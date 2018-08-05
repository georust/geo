use algorithm::map_coords::{MapCoords, MapCoordsInplace};
use CoordinateType;

pub trait Translate<T> {
    /// Translate a Geometry along its axes by the given offsets
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::translate::{Translate};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString::from(vec);
    /// let translated = linestring.translate(1.5, 3.5);
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(1.5, 3.5));
    /// correct.push(Point::new(6.5, 8.5));
    /// correct.push(Point::new(11.5, 13.5));
    /// let correct_ls = LineString::from(correct);
    /// assert_eq!(translated, correct_ls);
    /// ```
    fn translate(&self, xoff: T, yoff: T) -> Self
    where
        T: CoordinateType;

    /// Translate a Geometry along its axes, but in place.
    fn translate_inplace(&mut self, xoff: T, yoff: T)
    where
        T: CoordinateType;
}

impl<T, G> Translate<T> for G
where
    T: CoordinateType,
    G: MapCoords<T, T, Output = G> + MapCoordsInplace<T>,
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        self.map_coords(&|&(x, y)| (x + xoff, y + yoff))
    }

    fn translate_inplace(&mut self, xoff: T, yoff: T) {
        self.map_coords_inplace(&|&(x, y)| (x + xoff, y + yoff))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use {Coordinate, LineString, Point, Polygon};
    #[test]
    fn test_translate_point() {
        let p = Point::new(1.0, 5.0);
        let translated = p.translate(30.0, 20.0);
        assert_eq!(translated, Point::new(31.0, 25.0));
    }
    #[test]
    fn test_translate_linestring() {
        let linestring = LineString::from(vec![(0.0, 0.0), (5.0, 1.0), (10.0, 0.0)]);
        let translated = linestring.translate(17.0, 18.0);
        let mut correct = Vec::new();
        correct.push((17.0, 18.0));
        correct.push((22.0, 19.0));
        correct.push((27., 18.));
        let correct_ls = LineString::from(correct);
        assert_eq!(translated, correct_ls);
    }
    #[test]
    fn test_translate_polygon() {
        let points_raw = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);
        let translated = poly1.translate(17.0, 18.0);
        let correct_outside = vec![
            (22.0, 19.0),
            (21.0, 20.0),
            (21.0, 21.0),
            (22.0, 22.0),
            (23.0, 22.0),
            (24.0, 21.0),
            (24.0, 20.0),
            (23.0, 19.0),
            (22.0, 19.0),
        ];
        let correct = Polygon::new(
            LineString(
                correct_outside
                    .iter()
                    .map(|e| Coordinate { x: e.0, y: e.1 })
                    .collect::<Vec<_>>(),
            ),
            vec![],
        );
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
        assert_eq!(rotated.exterior, correct_outside);
        assert_eq!(rotated.interiors[0], correct_inside);
    }
}
