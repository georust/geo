use failure::Error;
use {
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect,
};

/// Map a function over all the coordinates in an object, returning a new one
pub trait MapCoords<T, NT> {
    type Output;

    /// Apply a function to all the coordinates in a geometric object, returning a new object.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::map_coords::MapCoords;
    ///
    /// let p1 = Point::new(10., 20.);
    /// let p2 = p1.map_coords(&|&(x, y)| (x+1000., y*2.));
    ///
    /// assert_eq!(p2, Point::new(1010., 40.));
    /// ```
    ///
    /// You can convert the coordinate type this way as well
    ///
    /// ```
    /// # use geo::Point;
    /// # use geo::algorithm::map_coords::MapCoords;
    ///
    /// let p1: Point<f32> = Point::new(10.0f32, 20.0f32);
    /// let p2: Point<f64> = p1.map_coords(&|&(x, y)| (x as f64, y as f64));
    ///
    /// assert_eq!(p2, Point::new(10.0f64, 20.0f64));
    /// ```
    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    where
        T: CoordinateType,
        NT: CoordinateType;
}

/// Map a fallible function over all the coordinates in a geometry, returning a Result
pub trait TryMapCoords<T, NT> {
    type Output;

    /// Map a fallible function over all the coordinates in a geometry, returning a Result
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::map_coords::TryMapCoords;
    ///
    /// let p1 = Point::new(10., 20.);
    /// let p2 = p1.try_map_coords(&|&(x, y)| Ok((x+1000., y*2.))).unwrap();
    ///
    /// assert_eq!(p2, Point::new(1010., 40.));
    /// ```
    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error>
    where
        T: CoordinateType,
        NT: CoordinateType;
}

/// Map all the coordinates in an object in place
pub trait MapCoordsInplace<T> {
    /// Apply a function to all the coordinates in a geometric object, in place
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::map_coords::MapCoordsInplace;
    ///
    /// let mut p = Point::new(10., 20.);
    /// p.map_coords_inplace(&|&(x, y)| (x+1000., y*2.));
    ///
    /// assert_eq!(p, Point::new(1010., 40.));
    /// ```
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T))
    where
        T: CoordinateType;
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Point<T> {
    type Output = Point<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        let new_point = func(&(self.0.x, self.0.y));
        Point::new(new_point.0, new_point.1)
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Point<T> {
    type Output = Point<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        let new_point = func(&(self.0.x, self.0.y))?;
        Ok(Point::new(new_point.0, new_point.1))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Point<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        let new_point = func(&(self.0.x, self.0.y));
        self.0.x = new_point.0;
        self.0.y = new_point.1;
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Line<T> {
    type Output = Line<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        Line::new(
            self.start_point().map_coords(func).0,
            self.end_point().map_coords(func).0,
        )
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Line<T> {
    type Output = Line<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(Line::new(
            self.start_point().try_map_coords(func)?.0,
            self.end_point().try_map_coords(func)?.0,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Line<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        let new_start = func(&(self.start.x, self.start.y));
        self.start.x = new_start.0;
        self.start.y = new_start.1;

        let new_end = func(&(self.end.x, self.end.y));
        self.end.x = new_end.0;
        self.end.y = new_end.1;
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for LineString<T> {
    type Output = LineString<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        LineString::from(
            self.points_iter()
                .map(|p| p.map_coords(func))
                .collect::<Vec<_>>(),
        )
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for LineString<T> {
    type Output = LineString<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(LineString::from(
            self.points_iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for LineString<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            let new_coords = func(&(p.x, p.y));
            p.x = new_coords.0;
            p.y = new_coords.1;
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Polygon<T> {
    type Output = Polygon<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        Polygon::new(
            self.exterior().map_coords(func),
            self.interiors()
                .iter()
                .map(|l| l.map_coords(func))
                .collect(),
        )
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Polygon<T> {
    type Output = Polygon<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(Polygon::new(
            self.exterior().try_map_coords(func)?,
            self.interiors()
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Polygon<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        self.exterior_mut(|line_string| {
            line_string.map_coords_inplace(func);
        });

        self.interiors_mut(|line_strings| {
            for line_string in line_strings {
                line_string.map_coords_inplace(func);
            }
        });
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for MultiPoint<T> {
    type Output = MultiPoint<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        MultiPoint(self.0.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiPoint<T> {
    type Output = MultiPoint<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(MultiPoint(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiPoint<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for MultiLineString<T> {
    type Output = MultiLineString<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        MultiLineString(self.0.iter().map(|l| l.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiLineString<T> {
    type Output = MultiLineString<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(MultiLineString(
            self.0
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiLineString<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for MultiPolygon<T> {
    type Output = MultiPolygon<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        MultiPolygon(self.0.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiPolygon<T> {
    type Output = MultiPolygon<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(MultiPolygon(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiPolygon<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Geometry<T> {
    type Output = Geometry<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        match *self {
            Geometry::Point(ref x) => Geometry::Point(x.map_coords(func)),
            Geometry::Line(ref x) => Geometry::Line(x.map_coords(func)),
            Geometry::LineString(ref x) => Geometry::LineString(x.map_coords(func)),
            Geometry::Polygon(ref x) => Geometry::Polygon(x.map_coords(func)),
            Geometry::MultiPoint(ref x) => Geometry::MultiPoint(x.map_coords(func)),
            Geometry::MultiLineString(ref x) => Geometry::MultiLineString(x.map_coords(func)),
            Geometry::MultiPolygon(ref x) => Geometry::MultiPolygon(x.map_coords(func)),
            Geometry::GeometryCollection(ref x) => Geometry::GeometryCollection(x.map_coords(func)),
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Geometry<T> {
    type Output = Geometry<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        match *self {
            Geometry::Point(ref x) => Ok(Geometry::Point(x.try_map_coords(func)?)),
            Geometry::Line(ref x) => Ok(Geometry::Line(x.try_map_coords(func)?)),
            Geometry::LineString(ref x) => Ok(Geometry::LineString(x.try_map_coords(func)?)),
            Geometry::Polygon(ref x) => Ok(Geometry::Polygon(x.try_map_coords(func)?)),
            Geometry::MultiPoint(ref x) => Ok(Geometry::MultiPoint(x.try_map_coords(func)?)),
            Geometry::MultiLineString(ref x) => {
                Ok(Geometry::MultiLineString(x.try_map_coords(func)?))
            }
            Geometry::MultiPolygon(ref x) => Ok(Geometry::MultiPolygon(x.try_map_coords(func)?)),
            Geometry::GeometryCollection(ref x) => {
                Ok(Geometry::GeometryCollection(x.try_map_coords(func)?))
            }
        }
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Geometry<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        match *self {
            Geometry::Point(ref mut x) => x.map_coords_inplace(func),
            Geometry::Line(ref mut x) => x.map_coords_inplace(func),
            Geometry::LineString(ref mut x) => x.map_coords_inplace(func),
            Geometry::Polygon(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiPoint(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiLineString(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiPolygon(ref mut x) => x.map_coords_inplace(func),
            Geometry::GeometryCollection(ref mut x) => x.map_coords_inplace(func),
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for GeometryCollection<T> {
    type Output = GeometryCollection<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        GeometryCollection(self.0.iter().map(|g| g.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for GeometryCollection<T> {
    type Output = GeometryCollection<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        Ok(GeometryCollection(
            self.0
                .iter()
                .map(|g| g.try_map_coords(func))
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for GeometryCollection<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Rect<T> {
    type Output = Rect<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output {
        let new_min = func(&(self.min.x, self.min.y));
        let new_max = func(&(self.max.x, self.max.y));
        Rect {
            min: Coordinate {
                x: new_min.0,
                y: new_min.1,
            },
            max: Coordinate {
                x: new_max.0,
                y: new_max.1,
            },
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Rect<T> {
    type Output = Rect<NT>;

    fn try_map_coords(
        &self,
        func: &Fn(&(T, T)) -> Result<(NT, NT), Error>,
    ) -> Result<Self::Output, Error> {
        let new_min = func(&(self.min.x, self.min.y))?;
        let new_max = func(&(self.max.x, self.max.y))?;
        Ok(Rect {
            min: Coordinate {
                x: new_min.0,
                y: new_min.1,
            },
            max: Coordinate {
                x: new_max.0,
                y: new_max.1,
            },
        })
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Rect<T> {
    fn map_coords_inplace(&mut self, func: &Fn(&(T, T)) -> (T, T)) {
        let new_min = func(&(self.min.x, self.min.y));
        let new_max = func(&(self.max.x, self.max.y));
        self.min.x = new_min.0;
        self.min.y = new_min.1;
        self.max.x = new_max.0;
        self.max.y = new_max.1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Coordinate;

    #[test]
    fn point() {
        let p = Point::new(10., 10.);
        let new_p = p.map_coords(&|&(x, y)| (x + 10., y + 100.));
        assert_eq!(new_p.x(), 20.);
        assert_eq!(new_p.y(), 110.);
    }

    #[test]
    fn point_inplace() {
        let mut p2 = Point::new(10f32, 10f32);
        p2.map_coords_inplace(&|&(x, y)| (x + 10., y + 100.));
        assert_eq!(p2.x(), 20.);
        assert_eq!(p2.y(), 110.);
    }

    #[test]
    fn line() {
        let line = Line::from([(0., 0.), (1., 2.)]);
        assert_eq!(
            line.map_coords(&|&(x, y)| (x * 2., y)),
            Line::from([(0., 0.), (2., 2.)])
        );
    }

    #[test]
    fn linestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2 = line1.map_coords(&|&(x, y)| (x + 10., y - 100.));
        assert_eq!(line2.0[0], Coordinate::from((10., -100.)));
        assert_eq!(line2.0[1], Coordinate::from((11., -98.)));
    }

    #[test]
    fn polygon() {
        let exterior = LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]);
        let interiors = vec![LineString::from(vec![
            (0.1, 0.1),
            (0.9, 0.9),
            (0.9, 0.1),
            (0.1, 0.1),
        ])];
        let p = Polygon::new(exterior, interiors);

        let p2 = p.map_coords(&|&(x, y)| (x + 10., y - 100.));

        let exterior2 =
            LineString::from(vec![(10., -100.), (11., -99.), (11., -100.), (10., -100.)]);
        let interiors2 = vec![LineString::from(vec![
            (10.1, -99.9),
            (10.9, -99.1),
            (10.9, -99.9),
            (10.1, -99.9),
        ])];
        let expected_p2 = Polygon::new(exterior2, interiors2);

        assert_eq!(p2, expected_p2);
    }

    #[test]
    fn multipoint() {
        let p1 = Point::new(10., 10.);
        let p2 = Point::new(0., -100.);
        let mp = MultiPoint(vec![p1, p2]);

        assert_eq!(
            mp.map_coords(&|&(x, y)| (x + 10., y + 100.)),
            MultiPoint(vec![Point::new(20., 110.), Point::new(10., 0.)])
        );
    }

    #[test]
    fn multilinestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2: LineString<f32> = LineString::from(vec![(-1., 0.), (0., 0.), (1., 2.)]);
        let mline = MultiLineString(vec![line1, line2]);
        let mline2 = mline.map_coords(&|&(x, y)| (x + 10., y - 100.));
        assert_eq!(
            mline2,
            MultiLineString(vec![
                LineString::from(vec![(10., -100.), (11., -98.)]),
                LineString::from(vec![(9., -100.), (10., -100.), (11., -98.)]),
            ])
        );
    }

    #[test]
    fn multipolygon() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.), (0., 0.)]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![
                (11., 11.),
                (20., 11.),
                (20., 20.),
                (11., 20.),
                (11., 11.),
            ]),
            vec![LineString::from(vec![
                (13., 13.),
                (13., 17.),
                (17., 17.),
                (17., 13.),
                (13., 13.),
            ])],
        );

        let mp = MultiPolygon(vec![poly1, poly2]);
        let mp2 = mp.map_coords(&|&(x, y)| (x * 2., y + 100.));
        assert_eq!(mp2.0.len(), 2);
        assert_eq!(
            mp2.0[0],
            Polygon::new(
                LineString::from(vec![
                    (0., 100.),
                    (20., 100.),
                    (20., 110.),
                    (0., 110.),
                    (0., 100.),
                ]),
                vec![]
            )
        );
        assert_eq!(
            mp2.0[1],
            Polygon::new(
                LineString::from(vec![
                    (22., 111.),
                    (40., 111.),
                    (40., 120.),
                    (22., 120.),
                    (22., 111.),
                ]),
                vec![LineString::from(vec![
                    (26., 113.),
                    (26., 117.),
                    (34., 117.),
                    (34., 113.),
                    (26., 113.),
                ])]
            )
        );
    }

    #[test]
    fn geometrycollection() {
        let p1 = Geometry::Point(Point::new(10., 10.));
        let line1 = Geometry::LineString(LineString::from(vec![(0., 0.), (1., 2.)]));

        let gc = GeometryCollection(vec![p1, line1]);

        assert_eq!(
            gc.map_coords(&|&(x, y)| (x + 10., y + 100.)),
            GeometryCollection(vec![
                Geometry::Point(Point::new(20., 110.)),
                Geometry::LineString(LineString::from(vec![(10., 100.), (11., 102.)])),
            ])
        );
    }

    #[test]
    fn convert_type() {
        let p1: Point<f64> = Point::new(1., 2.);
        let p2: Point<f32> = p1.map_coords(&|&(x, y)| (x as f32, y as f32));
        assert_eq!(p2.x(), 1f32);
        assert_eq!(p2.y(), 2f32);
    }

    #[cfg(feature = "use-proj")]
    #[test]
    fn test_fallible() {
        let f = |x: f64, y: f64| {
            if x != 2.0 {
                Ok((x * 2., y + 100.))
            } else {
                Err(format_err!("Ugh"))
            }
        };
        // this should produce an error
        let bad_ls: LineString<_> = vec![
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
        ]
        .into();
        // this should be fine
        let good_ls: LineString<_> = vec![
            Point::new(1.0, 1.0),
            Point::new(2.1, 2.0),
            Point::new(3.0, 3.0),
        ]
        .into();
        let bad = bad_ls.try_map_coords(&|&(x, y)| f(x, y));
        assert!(bad.is_err());
        let good = good_ls.try_map_coords(&|&(x, y)| f(x, y));
        assert!(good.is_ok());
        assert_eq!(
            good.unwrap(),
            vec![
                Point::new(2., 101.),
                Point::new(4.2, 102.),
                Point::new(6.0, 103.),
            ]
            .into()
        );
    }
}
