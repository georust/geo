//! # Advanced Example: Fallible Geometry coordinate conversion using `PROJ`
//!
#![cfg_attr(feature = "use-proj", doc = "```")]
#![cfg_attr(not(feature = "use-proj"), doc = "```ignore")]
//! // activate the [use-proj] feature in cargo.toml in order to access proj functions
//! use approx::assert_relative_eq;
//! use geo::{Coordinate, Point};
//! use geo::MapCoords;
//! use proj::{Coord, Proj, ProjError};
//! // GeoJSON uses the WGS 84 coordinate system
//! let from = "EPSG:4326";
//! // The NAD83 / California zone 6 (ftUS) coordinate system
//! let to = "EPSG:2230";
//! let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
//! let transform = |c: Coordinate<f64>| -> Result<_, ProjError> {
//!     // proj can accept Point, Coordinate, Tuple, and array values, returning a Result
//!     let shifted = to_feet.convert(c)?;
//!     Ok(shifted)
//! };
//! // 👽
//! let usa_m = Point::new(-115.797615, 37.2647978);
//! let usa_ft = usa_m.try_map_coords(|coord| transform(coord)).unwrap();
//! assert_relative_eq!(6693625.67217475, usa_ft.x(), epsilon = 1e-6);
//! assert_relative_eq!(3497301.5918027186, usa_ft.y(), epsilon = 1e-6);
//! ```

pub use modern::*;
mod modern {
    pub(crate) use crate::{
        CoordNum, Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString,
        MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
    };

    /// Map a function over all the coordinates in an object, returning a new one
    pub trait MapCoords<T, NT> {
        type Output;

        /// Apply a function to all the coordinates in a geometric object, returning a new object.
        ///
        /// # Examples
        ///
        /// ```
        /// use geo::MapCoords;
        /// use geo::{Coordinate, Point};
        /// use approx::assert_relative_eq;
        ///
        /// let p1 = Point::new(10., 20.);
        /// let p2 = p1.map_coords(|Coordinate { x, y }| Coordinate { x: x + 1000., y: y * 2. });
        ///
        /// assert_relative_eq!(p2, Point::new(1010., 40.), epsilon = 1e-6);
        /// ```
        ///
        /// Note that the input and output numeric types need not match.
        ///
        /// For example, consider OpenStreetMap's coordinate encoding scheme, which, to save space,
        /// encodes latitude/longitude as 32bit signed integers from the floating point values
        /// to six decimal places (eg. lat/lon * 1000000).
        ///
        /// ```
        /// # use geo::{Coordinate, Point};
        /// # use geo::MapCoords;
        /// # use approx::assert_relative_eq;
        ///
        /// let SCALE_FACTOR: f64 = 1000000.0;
        ///
        /// let floating_point_geom: Point<f64> = Point::new(10.15f64, 20.05f64);
        /// let fixed_point_geom: Point<i32> = floating_point_geom.map_coords(|Coordinate { x, y }| {
        ///     Coordinate { x: (x * SCALE_FACTOR) as i32, y: (y * SCALE_FACTOR) as i32 }
        /// });
        ///
        /// assert_eq!(fixed_point_geom.x(), 10150000);
        /// ```
        ///
        /// If you want *only* to convert between numeric types (i32 -> f64) without further
        /// transformation, consider using [`Convert].
        fn map_coords(&self, func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy) -> Self::Output
        where
            T: CoordNum,
            NT: CoordNum;

        /// Map a fallible function over all the coordinates in a geometry, returning a Result
        ///
        /// # Examples
        ///
        /// ```
        /// use approx::assert_relative_eq;
        /// use geo::MapCoords;
        /// use geo::{Coordinate, Point};
        ///
        /// let p1 = Point::new(10., 20.);
        /// let p2 = p1
        ///     .try_map_coords(|Coordinate { x, y }| -> Result<_, std::convert::Infallible> {
        ///         Ok(Coordinate { x: x + 1000., y: y * 2. })
        ///     }).unwrap();
        ///
        /// assert_relative_eq!(p2, Point::new(1010., 40.), epsilon = 1e-6);
        /// ```
        ///
        /// ## Advanced Example: Geometry coordinate conversion using `PROJ`
        ///
        #[cfg_attr(feature = "use-proj", doc = "```")]
        #[cfg_attr(not(feature = "use-proj"), doc = "```ignore")]
        /// use approx::assert_relative_eq;
        /// // activate the [use-proj] feature in cargo.toml in order to access proj functions
        /// use geo::{Coordinate, Point};
        /// use geo::map_coords::MapCoords;
        /// use proj::{Coord, Proj, ProjError};
        /// // GeoJSON uses the WGS 84 coordinate system
        /// let from = "EPSG:4326";
        /// // The NAD83 / California zone 6 (ftUS) coordinate system
        /// let to = "EPSG:2230";
        /// let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
        /// let transform = |c: Coordinate<f64>| -> Result<_, ProjError> {
        ///     // proj can accept Point, Coordinate, Tuple, and array values, returning a Result
        ///     let shifted = to_feet.convert(c)?;
        ///     Ok(shifted)
        /// };
        /// // 👽
        /// let usa_m = Point::new(-115.797615, 37.2647978);
        /// let usa_ft = usa_m.try_map_coords(|coord| transform(coord)).unwrap();
        /// assert_relative_eq!(6693625.67217475, usa_ft.x(), epsilon = 1e-6);
        /// assert_relative_eq!(3497301.5918027186, usa_ft.y(), epsilon = 1e-6);
        /// ```
        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E>
        where
            T: CoordNum,
            NT: CoordNum;
    }

    pub trait MapCoordsInPlace<T> {
        /// Apply a function to all the coordinates in a geometric object, in place
        ///
        /// # Examples
        ///
        /// ```
        /// use geo::MapCoordsInPlace;
        /// use geo::{Coordinate, Point};
        /// use approx::assert_relative_eq;
        ///
        /// let mut p = Point::new(10., 20.);
        /// p.map_coords_in_place(|Coordinate { x, y }| Coordinate { x: x + 1000., y: y * 2. });
        ///
        /// assert_relative_eq!(p, Point::new(1010., 40.), epsilon = 1e-6);
        /// ```
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy)
        where
            T: CoordNum;

        /// Map a fallible function over all the coordinates in a geometry, in place, returning a `Result`.
        ///
        /// Upon encountering an `Err` from the function, `try_map_coords_in_place` immediately returns
        /// and the geometry is potentially left in a partially mapped state.
        ///
        /// # Examples
        ///
        /// ```
        /// use geo::MapCoordsInPlace;
        /// use geo::Coordinate;
        ///
        /// let mut p1 = geo::point!{x: 10u32, y: 20u32};
        ///
        /// p1.try_map_coords_in_place(|Coordinate { x, y }| -> Result<_, &str> {
        ///     Ok(Coordinate {
        ///         x: x.checked_add(1000).ok_or("Overflow")?,
        ///         y: y.checked_mul(2).ok_or("Overflow")?,
        ///     })
        /// })?;
        ///
        /// assert_eq!(
        ///     p1,
        ///     geo::point!{x: 1010u32, y: 40u32},
        /// );
        /// # Ok::<(), &str>(())
        /// ```
        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E>
        where
            T: CoordNum;
    }

    //-----------------------//
    // Point implementations //
    //-----------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Point<T> {
        type Output = Point<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            Point(func(self.0))
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E>,
        ) -> Result<Self::Output, E> {
            Ok(Point(func(self.0)?))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Point<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T>) {
            self.0 = func(self.0);
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            self.0 = func(self.0)?;
            Ok(())
        }
    }

    //----------------------//
    // Line implementations //
    //----------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Line<T> {
        type Output = Line<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            Line::new(
                self.start_point().map_coords(func).0,
                self.end_point().map_coords(func).0,
            )
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(Line::new(
                self.start_point().try_map_coords(func)?.0,
                self.end_point().try_map_coords(func)?.0,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Line<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T>) {
            self.start = func(self.start);
            self.end = func(self.end);
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            self.start = func(self.start)?;
            self.end = func(self.end)?;

            Ok(())
        }
    }

    //----------------------------//
    // LineString implementations //
    //----------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for LineString<T> {
        type Output = LineString<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            LineString::from(
                self.points()
                    .map(|p| p.map_coords(func))
                    .collect::<Vec<_>>(),
            )
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(LineString::from(
                self.points()
                    .map(|p| p.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for LineString<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T>) {
            for p in &mut self.0 {
                *p = func(*p);
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            for p in &mut self.0 {
                *p = func(*p)?;
            }
            Ok(())
        }
    }

    //-------------------------//
    // Polygon implementations //
    //-------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Polygon<T> {
        type Output = Polygon<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            Polygon::new(
                self.exterior().map_coords(func),
                self.interiors()
                    .iter()
                    .map(|l| l.map_coords(func))
                    .collect(),
            )
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(Polygon::new(
                self.exterior().try_map_coords(func)?,
                self.interiors()
                    .iter()
                    .map(|l| l.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Polygon<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            self.exterior_mut(|line_string| {
                line_string.map_coords_in_place(func);
            });

            self.interiors_mut(|line_strings| {
                for line_string in line_strings {
                    line_string.map_coords_in_place(func);
                }
            });
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            let mut result = Ok(());

            self.exterior_mut(|line_string| {
                if let Err(e) = line_string.try_map_coords_in_place(&func) {
                    result = Err(e);
                }
            });

            if result.is_ok() {
                self.interiors_mut(|line_strings| {
                    for line_string in line_strings {
                        if let Err(e) = line_string.try_map_coords_in_place(&func) {
                            result = Err(e);
                            break;
                        }
                    }
                });
            }

            result
        }
    }

    //----------------------------//
    // MultiPoint implementations //
    //----------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for MultiPoint<T> {
        type Output = MultiPoint<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            MultiPoint::new(self.iter().map(|p| p.map_coords(func)).collect())
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(MultiPoint::new(
                self.0
                    .iter()
                    .map(|p| p.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for MultiPoint<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            for p in &mut self.0 {
                p.map_coords_in_place(func);
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            for p in &mut self.0 {
                p.try_map_coords_in_place(&func)?;
            }
            Ok(())
        }
    }

    //---------------------------------//
    // MultiLineString implementations //
    //---------------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for MultiLineString<T> {
        type Output = MultiLineString<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            MultiLineString::new(self.iter().map(|l| l.map_coords(func)).collect())
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(MultiLineString::new(
                self.0
                    .iter()
                    .map(|l| l.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for MultiLineString<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            for p in &mut self.0 {
                p.map_coords_in_place(func);
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            for p in &mut self.0 {
                p.try_map_coords_in_place(&func)?;
            }
            Ok(())
        }
    }

    //------------------------------//
    // MultiPolygon implementations //
    //------------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for MultiPolygon<T> {
        type Output = MultiPolygon<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            MultiPolygon::new(self.iter().map(|p| p.map_coords(func)).collect())
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(MultiPolygon::new(
                self.0
                    .iter()
                    .map(|p| p.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for MultiPolygon<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            for p in &mut self.0 {
                p.map_coords_in_place(func);
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            for p in &mut self.0 {
                p.try_map_coords_in_place(&func)?;
            }
            Ok(())
        }
    }

    //--------------------------//
    // Geometry implementations //
    //--------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Geometry<T> {
        type Output = Geometry<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            match *self {
                Geometry::Point(ref x) => Geometry::Point(x.map_coords(func)),
                Geometry::Line(ref x) => Geometry::Line(x.map_coords(func)),
                Geometry::LineString(ref x) => Geometry::LineString(x.map_coords(func)),
                Geometry::Polygon(ref x) => Geometry::Polygon(x.map_coords(func)),
                Geometry::MultiPoint(ref x) => Geometry::MultiPoint(x.map_coords(func)),
                Geometry::MultiLineString(ref x) => Geometry::MultiLineString(x.map_coords(func)),
                Geometry::MultiPolygon(ref x) => Geometry::MultiPolygon(x.map_coords(func)),
                Geometry::GeometryCollection(ref x) => {
                    Geometry::GeometryCollection(x.map_coords(func))
                }
                Geometry::Rect(ref x) => Geometry::Rect(x.map_coords(func)),
                Geometry::Triangle(ref x) => Geometry::Triangle(x.map_coords(func)),
            }
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            match *self {
                Geometry::Point(ref x) => Ok(Geometry::Point(x.try_map_coords(func)?)),
                Geometry::Line(ref x) => Ok(Geometry::Line(x.try_map_coords(func)?)),
                Geometry::LineString(ref x) => Ok(Geometry::LineString(x.try_map_coords(func)?)),
                Geometry::Polygon(ref x) => Ok(Geometry::Polygon(x.try_map_coords(func)?)),
                Geometry::MultiPoint(ref x) => Ok(Geometry::MultiPoint(x.try_map_coords(func)?)),
                Geometry::MultiLineString(ref x) => {
                    Ok(Geometry::MultiLineString(x.try_map_coords(func)?))
                }
                Geometry::MultiPolygon(ref x) => {
                    Ok(Geometry::MultiPolygon(x.try_map_coords(func)?))
                }
                Geometry::GeometryCollection(ref x) => {
                    Ok(Geometry::GeometryCollection(x.try_map_coords(func)?))
                }
                Geometry::Rect(ref x) => Ok(Geometry::Rect(x.try_map_coords(func)?)),
                Geometry::Triangle(ref x) => Ok(Geometry::Triangle(x.try_map_coords(func)?)),
            }
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Geometry<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            match *self {
                Geometry::Point(ref mut x) => x.map_coords_in_place(func),
                Geometry::Line(ref mut x) => x.map_coords_in_place(func),
                Geometry::LineString(ref mut x) => x.map_coords_in_place(func),
                Geometry::Polygon(ref mut x) => x.map_coords_in_place(func),
                Geometry::MultiPoint(ref mut x) => x.map_coords_in_place(func),
                Geometry::MultiLineString(ref mut x) => x.map_coords_in_place(func),
                Geometry::MultiPolygon(ref mut x) => x.map_coords_in_place(func),
                Geometry::GeometryCollection(ref mut x) => x.map_coords_in_place(func),
                Geometry::Rect(ref mut x) => x.map_coords_in_place(func),
                Geometry::Triangle(ref mut x) => x.map_coords_in_place(func),
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            match *self {
                Geometry::Point(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::Line(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::LineString(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::Polygon(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::MultiPoint(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::MultiLineString(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::MultiPolygon(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::GeometryCollection(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::Rect(ref mut x) => x.try_map_coords_in_place(func),
                Geometry::Triangle(ref mut x) => x.try_map_coords_in_place(func),
            }
        }
    }

    //------------------------------------//
    // GeometryCollection implementations //
    //------------------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for GeometryCollection<T> {
        type Output = GeometryCollection<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            GeometryCollection::new_from(self.iter().map(|g| g.map_coords(func)).collect())
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E> + Copy,
        ) -> Result<Self::Output, E> {
            Ok(GeometryCollection::new_from(
                self.0
                    .iter()
                    .map(|g| g.try_map_coords(func))
                    .collect::<Result<Vec<_>, E>>()?,
            ))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for GeometryCollection<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T> + Copy) {
            for p in &mut self.0 {
                p.map_coords_in_place(func);
            }
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            for p in &mut self.0 {
                p.try_map_coords_in_place(&func)?;
            }
            Ok(())
        }
    }

    //----------------------//
    // Rect implementations //
    //----------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Rect<T> {
        type Output = Rect<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            Rect::new(func(self.min()), func(self.max()))
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E>,
        ) -> Result<Self::Output, E> {
            Ok(Rect::new(func(self.min())?, func(self.max())?))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Rect<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T>) {
            let mut new_rect = Rect::new(func(self.min()), func(self.max()));
            ::std::mem::swap(self, &mut new_rect);
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            let mut new_rect = Rect::new(func(self.min())?, func(self.max())?);
            ::std::mem::swap(self, &mut new_rect);
            Ok(())
        }
    }

    //--------------------------//
    // Triangle implementations //
    //--------------------------//

    impl<T: CoordNum, NT: CoordNum> MapCoords<T, NT> for Triangle<T> {
        type Output = Triangle<NT>;

        fn map_coords(
            &self,
            func: impl Fn(Coordinate<T>) -> Coordinate<NT> + Copy,
        ) -> Self::Output {
            Triangle::new(func(self.0), func(self.1), func(self.2))
        }

        fn try_map_coords<E>(
            &self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<NT>, E>,
        ) -> Result<Self::Output, E> {
            Ok(Triangle::new(func(self.0)?, func(self.1)?, func(self.2)?))
        }
    }

    impl<T: CoordNum> MapCoordsInPlace<T> for Triangle<T> {
        fn map_coords_in_place(&mut self, func: impl Fn(Coordinate<T>) -> Coordinate<T>) {
            let mut new_triangle = Triangle::new(func(self.0), func(self.1), func(self.2));

            ::std::mem::swap(self, &mut new_triangle);
        }

        fn try_map_coords_in_place<E>(
            &mut self,
            func: impl Fn(Coordinate<T>) -> Result<Coordinate<T>, E>,
        ) -> Result<(), E> {
            let mut new_triangle = Triangle::new(func(self.0)?, func(self.1)?, func(self.2)?);

            ::std::mem::swap(self, &mut new_triangle);

            Ok(())
        }
    }
}
pub use deprecated::*;
pub(crate) mod deprecated {
    use super::*;

    /// Map a fallible function over all the coordinates in a geometry, returning a Result
    #[deprecated(
        since = "0.21.0",
        note = "use `MapCoords::try_map_coords` which takes a `Coordinate` instead of an (x,y) tuple"
    )]
    pub trait TryMapCoords<T, NT, E> {
        type Output;

        /// Map a fallible function over all the coordinates in a geometry, returning a Result
        ///
        /// # Examples
        ///
        /// ```
        /// use approx::assert_relative_eq;
        /// #[allow(deprecated)]
        /// use geo::TryMapCoords;
        /// use geo::Point;
        ///
        /// let p1 = Point::new(10., 20.);
        /// #[allow(deprecated)]
        /// let p2 = p1
        ///     .try_map_coords(|(x, y)| -> Result<_, std::convert::Infallible> {
        ///         Ok((x + 1000., y * 2.))
        ///     }).unwrap();
        ///
        /// assert_relative_eq!(p2, Point::new(1010., 40.), epsilon = 1e-6);
        /// ```
        ///
        /// ## Advanced Example: Geometry coordinate conversion using `PROJ`
        ///
        #[cfg_attr(feature = "use-proj", doc = "```")]
        #[cfg_attr(not(feature = "use-proj"), doc = "```ignore")]
        /// use approx::assert_relative_eq;
        /// // activate the [use-proj] feature in cargo.toml in order to access proj functions
        /// use geo::{Coordinate, Point};
        /// #[allow(deprecated)]
        /// use geo::TryMapCoords;
        /// use proj::{Coord, Proj, ProjError};
        /// // GeoJSON uses the WGS 84 coordinate system
        /// let from = "EPSG:4326";
        /// // The NAD83 / California zone 6 (ftUS) coordinate system
        /// let to = "EPSG:2230";
        /// let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
        /// let f = |x: f64, y: f64| -> Result<_, ProjError> {
        ///     // proj can accept Point, Coordinate, Tuple, and array values, returning a Result
        ///     let shifted = to_feet.convert((x, y))?;
        ///     Ok((shifted.x(), shifted.y()))
        /// };
        ///
        /// // 👽
        /// let usa_m = Point::new(-115.797615, 37.2647978);
        /// #[allow(deprecated)]
        /// let usa_ft = usa_m.try_map_coords(|(x, y)| f(x, y)).unwrap();
        /// assert_relative_eq!(6693625.67217475, usa_ft.x(), epsilon = 1e-6);
        /// assert_relative_eq!(3497301.5918027186, usa_ft.y(), epsilon = 1e-6);
        /// ```
        fn try_map_coords(
            &self,
            func: impl Fn((T, T)) -> Result<(NT, NT), E> + Copy,
        ) -> Result<Self::Output, E>
        where
            T: CoordNum,
            NT: CoordNum;
    }

    #[deprecated(
        since = "0.21.0",
        note = "use `MapCoordsInPlace::try_map_coords_in_place` which takes a `Coordinate` instead of an (x,y) tuple"
    )]
    pub trait TryMapCoordsInplace<T, E> {
        /// Map a fallible function over all the coordinates in a geometry, in place, returning a `Result`.
        ///
        /// Upon encountering an `Err` from the function, `try_map_coords_in_place` immediately returns
        /// and the geometry is potentially left in a partially mapped state.
        ///
        /// # Examples
        ///
        /// ```
        /// #[allow(deprecated)]
        /// use geo::TryMapCoordsInplace;
        ///
        /// let mut p1 = geo::point!{x: 10u32, y: 20u32};
        ///
        /// #[allow(deprecated)]
        /// p1.try_map_coords_inplace(|(x, y)| -> Result<_, &str> {
        ///     Ok((
        ///         x.checked_add(1000).ok_or("Overflow")?,
        ///         y.checked_mul(2).ok_or("Overflow")?,
        ///     ))
        /// })?;
        ///
        /// assert_eq!(
        ///     p1,
        ///     geo::point!{x: 1010u32, y: 40u32},
        /// );
        /// # Ok::<(), &str>(())
        /// ```
        fn try_map_coords_inplace(
            &mut self,
            func: impl Fn((T, T)) -> Result<(T, T), E>,
        ) -> Result<(), E>
        where
            T: CoordNum;
    }

    /// Map a function over all the coordinates in an object in place
    #[deprecated(
        since = "0.21.0",
        note = "use `MapCoordsInPlace::map_coords_in_place` instead which takes a `Coordinate` instead of an (x,y) tuple"
    )]
    pub trait MapCoordsInplace<T>: MapCoordsInPlace<T> {
        /// Apply a function to all the coordinates in a geometric object, in place
        ///
        /// # Examples
        ///
        /// ```
        /// #[allow(deprecated)]
        /// use geo::MapCoordsInplace;
        /// use geo::Point;
        /// use approx::assert_relative_eq;
        ///
        /// let mut p = Point::new(10., 20.);
        /// #[allow(deprecated)]
        /// p.map_coords_inplace(|(x, y)| (x + 1000., y * 2.));
        ///
        /// assert_relative_eq!(p, Point::new(1010., 40.), epsilon = 1e-6);
        /// ```
        fn map_coords_inplace(&mut self, func: impl Fn((T, T)) -> (T, T) + Copy)
        where
            T: CoordNum;
    }

    macro_rules! impl_deprecated_map_coords {
        ($geom:ident) => {
            #[allow(deprecated)]
            impl<T: CoordNum, NT: CoordNum, E> TryMapCoords<T, NT, E> for $geom<T> {
                type Output = $geom<NT>;

                fn try_map_coords(
                    &self,
                    func: impl Fn((T, T)) -> Result<(NT, NT), E> + Copy,
                ) -> Result<Self::Output, E> {
                    MapCoords::try_map_coords(self, |c| Ok(Coordinate::from(func(c.x_y())?)))
                }
            }

            #[allow(deprecated)]
            impl<T: CoordNum, E> TryMapCoordsInplace<T, E> for $geom<T> {
                fn try_map_coords_inplace(
                    &mut self,
                    func: impl Fn((T, T)) -> Result<(T, T), E>,
                ) -> Result<(), E> {
                    MapCoordsInPlace::try_map_coords_in_place(self, |c| Ok(func(c.x_y())?.into()))
                }
            }

            #[allow(deprecated)]
            impl<T: CoordNum> MapCoordsInplace<T> for $geom<T> {
                /// Apply a function to all the coordinates in a geometric object, in place
                ///
                /// # Examples
                ///
                /// ```
                /// #[allow(deprecated)]
                /// use geo::MapCoordsInplace;
                /// use geo::Point;
                /// use approx::assert_relative_eq;
                ///
                /// let mut p = Point::new(10., 20.);
                /// #[allow(deprecated)]
                /// p.map_coords_inplace(|(x, y)| (x + 1000., y * 2.));
                ///
                /// assert_relative_eq!(p, Point::new(1010., 40.), epsilon = 1e-6);
                /// ```
                fn map_coords_inplace(&mut self, func: impl Fn((T, T)) -> (T, T) + Copy)
                where
                    T: CoordNum,
                {
                    MapCoordsInPlace::map_coords_in_place(self, |c| func(c.x_y()).into())
                }
            }
        };
    }

    impl_deprecated_map_coords!(Point);
    impl_deprecated_map_coords!(Line);
    impl_deprecated_map_coords!(LineString);
    impl_deprecated_map_coords!(Polygon);
    impl_deprecated_map_coords!(MultiPoint);
    impl_deprecated_map_coords!(MultiLineString);
    impl_deprecated_map_coords!(MultiPolygon);
    impl_deprecated_map_coords!(Geometry);
    impl_deprecated_map_coords!(GeometryCollection);
    impl_deprecated_map_coords!(Triangle);
    impl_deprecated_map_coords!(Rect);
}

#[cfg(test)]
mod test {
    use super::{MapCoords, MapCoordsInPlace};
    use crate::{
        coord, polygon, Coordinate, Geometry, GeometryCollection, Line, LineString,
        MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect,
    };

    #[test]
    fn point() {
        let p = Point::new(10., 10.);
        let new_p = p.map_coords(|Coordinate { x, y }| (x + 10., y + 100.).into());
        assert_relative_eq!(new_p.x(), 20.);
        assert_relative_eq!(new_p.y(), 110.);
    }

    #[test]
    fn point_inplace() {
        let mut p2 = Point::new(10f32, 10f32);
        p2.map_coords_in_place(|Coordinate { x, y }| (x + 10., y + 100.).into());
        assert_relative_eq!(p2.x(), 20.);
        assert_relative_eq!(p2.y(), 110.);
    }

    #[test]
    fn rect_inplace() {
        let mut rect = Rect::new((10, 10), (20, 20));
        rect.map_coords_in_place(|Coordinate { x, y }| (x + 10, y + 20).into());
        assert_eq!(rect.min(), coord! { x: 20, y: 30 });
        assert_eq!(rect.max(), coord! { x: 30, y: 40 });
    }

    #[test]
    fn rect_inplace_normalized() {
        let mut rect = Rect::new((2, 2), (3, 3));
        // Rect's enforce that rect.min is up and left of p2.  Here we test that the points are
        // normalized into a valid rect, regardless of the order they are mapped.
        rect.map_coords_in_place(|pt| {
            match pt.x_y() {
                // old min point maps to new max point
                (2, 2) => (4, 4).into(),
                // old max point maps to new min point
                (3, 3) => (1, 1).into(),
                _ => panic!("unexpected point"),
            }
        });

        assert_eq!(rect.min(), coord! { x: 1, y: 1 });
        assert_eq!(rect.max(), coord! { x: 4, y: 4 });
    }

    #[test]
    fn rect_map_coords() {
        let rect = Rect::new((10, 10), (20, 20));
        let another_rect = rect.map_coords(|Coordinate { x, y }| (x + 10, y + 20).into());
        assert_eq!(another_rect.min(), coord! { x: 20, y: 30 });
        assert_eq!(another_rect.max(), coord! { x: 30, y: 40 });
    }

    #[test]
    fn rect_try_map_coords() {
        let rect = Rect::new((10i32, 10), (20, 20));
        let result = rect.try_map_coords(|Coordinate { x, y }| -> Result<_, &'static str> {
            Ok((
                x.checked_add(10).ok_or("overflow")?,
                y.checked_add(20).ok_or("overflow")?,
            )
                .into())
        });
        assert!(result.is_ok());
    }

    #[test]
    fn rect_try_map_coords_normalized() {
        let rect = Rect::new((2, 2), (3, 3));
        // Rect's enforce that rect.min is up and left of p2.  Here we test that the points are
        // normalized into a valid rect, regardless of the order they are mapped.
        let result: Result<_, std::convert::Infallible> = rect.try_map_coords(|pt| {
            match pt.x_y() {
                // old min point maps to new max point
                (2, 2) => Ok((4, 4).into()),
                // old max point maps to new min point
                (3, 3) => Ok((1, 1).into()),
                _ => panic!("unexpected point"),
            }
        });
        let new_rect = result.unwrap();
        assert_eq!(new_rect.min(), coord! { x: 1, y: 1 });
        assert_eq!(new_rect.max(), coord! { x: 4, y: 4 });
    }

    #[test]
    fn line() {
        let line = Line::from([(0., 0.), (1., 2.)]);
        assert_relative_eq!(
            line.map_coords(|Coordinate { x, y }| (x * 2., y).into()),
            Line::from([(0., 0.), (2., 2.)]),
            epsilon = 1e-6
        );
    }

    #[test]
    fn linestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2 = line1.map_coords(|Coordinate { x, y }| (x + 10., y - 100.).into());
        assert_relative_eq!(line2.0[0], Coordinate::from((10., -100.)), epsilon = 1e-6);
        assert_relative_eq!(line2.0[1], Coordinate::from((11., -98.)), epsilon = 1e-6);
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

        let p2 = p.map_coords(|Coordinate { x, y }| (x + 10., y - 100.).into());

        let exterior2 =
            LineString::from(vec![(10., -100.), (11., -99.), (11., -100.), (10., -100.)]);
        let interiors2 = vec![LineString::from(vec![
            (10.1, -99.9),
            (10.9, -99.1),
            (10.9, -99.9),
            (10.1, -99.9),
        ])];
        let expected_p2 = Polygon::new(exterior2, interiors2);

        assert_relative_eq!(p2, expected_p2, epsilon = 1e-6);
    }

    #[test]
    fn multipoint() {
        let p1 = Point::new(10., 10.);
        let p2 = Point::new(0., -100.);
        let mp = MultiPoint::new(vec![p1, p2]);

        assert_eq!(
            mp.map_coords(|Coordinate { x, y }| (x + 10., y + 100.).into()),
            MultiPoint::new(vec![Point::new(20., 110.), Point::new(10., 0.)])
        );
    }

    #[test]
    fn multilinestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2: LineString<f32> = LineString::from(vec![(-1., 0.), (0., 0.), (1., 2.)]);
        let mline = MultiLineString::new(vec![line1, line2]);
        let mline2 = mline.map_coords(|Coordinate { x, y }| (x + 10., y - 100.).into());
        assert_relative_eq!(
            mline2,
            MultiLineString::new(vec![
                LineString::from(vec![(10., -100.), (11., -98.)]),
                LineString::from(vec![(9., -100.), (10., -100.), (11., -98.)]),
            ]),
            epsilon = 1e-6
        );
    }

    #[test]
    fn multipolygon() {
        let poly1 = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.),
        ];
        let poly2 = polygon![
            exterior: [
                (x: 11., y: 11.),
                (x: 20., y: 11.),
                (x: 20., y: 20.),
                (x: 11., y: 20.),
                (x: 11., y: 11.),
            ],
            interiors: [
                [
                    (x: 13., y: 13.),
                    (x: 13., y: 17.),
                    (x: 17., y: 17.),
                    (x: 17., y: 13.),
                    (x: 13., y: 13.),
                ]
            ],
        ];

        let mp = MultiPolygon::new(vec![poly1, poly2]);
        let mp2 = mp.map_coords(|Coordinate { x, y }| (x * 2., y + 100.).into());
        assert_eq!(mp2.0.len(), 2);
        assert_relative_eq!(
            mp2.0[0],
            polygon![
                (x: 0., y: 100.),
                (x: 20., y: 100.),
                (x: 20., y: 110.),
                (x: 0., y: 110.),
                (x: 0., y: 100.),
            ],
        );
        assert_relative_eq!(
            mp2.0[1],
            polygon![
                exterior: [
                    (x: 22., y: 111.),
                    (x: 40., y: 111.),
                    (x: 40., y: 120.),
                    (x: 22., y: 120.),
                    (x: 22., y: 111.),
                ],
                interiors: [
                    [
                        (x: 26., y: 113.),
                        (x: 26., y: 117.),
                        (x: 34., y: 117.),
                        (x: 34., y: 113.),
                        (x: 26., y: 113.),
                    ],
                ],
            ],
        );
    }

    #[test]
    fn geometrycollection() {
        let p1 = Geometry::Point(Point::new(10., 10.));
        let line1 = Geometry::LineString(LineString::from(vec![(0., 0.), (1., 2.)]));

        let gc = GeometryCollection::new_from(vec![p1, line1]);

        assert_eq!(
            gc.map_coords(|Coordinate { x, y }| (x + 10., y + 100.).into()),
            GeometryCollection::new_from(vec![
                Geometry::Point(Point::new(20., 110.)),
                Geometry::LineString(LineString::from(vec![(10., 100.), (11., 102.)])),
            ])
        );
    }

    #[test]
    fn convert_type() {
        let p1: Point<f64> = Point::new(1., 2.);
        let p2: Point<f32> = p1.map_coords(|Coordinate { x, y }| (x as f32, y as f32).into());
        assert_relative_eq!(p2.x(), 1f32);
        assert_relative_eq!(p2.y(), 2f32);
    }

    #[cfg(feature = "use-proj")]
    #[test]
    fn test_fallible_proj() {
        use proj::{Proj, ProjError};
        let from = "EPSG:4326";
        let to = "EPSG:2230";
        let to_feet = Proj::new_known_crs(from, to, None).unwrap();

        let f = |c| -> Result<_, ProjError> {
            let shifted = to_feet.convert(c)?;
            Ok(shifted)
        };
        // 👽
        let usa_m = Point::new(-115.797615, 37.2647978);
        let usa_ft = usa_m.try_map_coords(f).unwrap();
        assert_relative_eq!(6693625.67217475, usa_ft.x(), epsilon = 1e-6);
        assert_relative_eq!(3497301.5918027186, usa_ft.y(), epsilon = 1e-6);
    }

    #[test]
    fn test_fallible() {
        let f = |Coordinate { x, y }| -> Result<_, &'static str> {
            if relative_ne!(x, 2.0) {
                Ok((x * 2., y + 100.).into())
            } else {
                Err("Ugh")
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
        let bad = bad_ls.try_map_coords(f);
        assert!(bad.is_err());
        let good = good_ls.try_map_coords(f);
        assert!(good.is_ok());
        assert_relative_eq!(
            good.unwrap(),
            vec![
                Point::new(2., 101.),
                Point::new(4.2, 102.),
                Point::new(6.0, 103.),
            ]
            .into()
        );
    }

    #[test]
    fn rect_map_invert_coords() {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. });

        // This call should not panic even though Rect::new
        // constructor panics if min coords > max coords
        rect.map_coords(|Coordinate { x, y }| (-x, -y).into());
    }
}
