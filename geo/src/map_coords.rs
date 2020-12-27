//! # Advanced Example: Fallible Geometry coordinate conversion using `PROJ`
//!
//! ```
//! // activate the [use-proj] feature in cargo.toml in order to access proj functions
//! # #[cfg(feature = "use-proj")]
//! use geo::{Coordinate, Point};
//! # #[cfg(feature = "use-proj")]
//! use geo::algorithm::map_coords::TryMapCoords;
//! # #[cfg(feature = "use-proj")]
//! use proj::Proj;
//! // GeoJSON uses the WGS 84 coordinate system
//! # #[cfg(feature = "use-proj")]
//! let from = "EPSG:4326";
//! // The NAD83 / California zone 6 (ftUS) coordinate system
//! # #[cfg(feature = "use-proj")]
//! let to = "EPSG:2230";
//! # #[cfg(feature = "use-proj")]
//! let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
//! # #[cfg(feature = "use-proj")]
//! let f = |x: f64, y: f64| {
//!     // proj can accept Point, Coordinate, Tuple, and array values, returning a Result
//!     let shifted = to_feet.convert((x, y))?;
//!     Ok((shifted.x(), shifted.y()))
//! };
//! # #[cfg(feature = "use-proj")]
//! // 👽
//! # #[cfg(feature = "use-proj")]
//! let usa_m = Point::new(-115.797615, 37.2647978);
//! # #[cfg(feature = "use-proj")]
//! let usa_ft = usa_m.try_map_coords(|&(x, y)| f(x, y)).unwrap();
//! # #[cfg(feature = "use-proj")]
//! assert_eq!(6693625.67217475, usa_ft.x());
//! # #[cfg(feature = "use-proj")]
//! assert_eq!(3497301.5918027186, usa_ft.y());
//! ```

use crate::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::error::Error;

/// Map a function over all the coordinates in an object, returning a new one
pub trait MapCoords<T, NT> {
    type Output;

    /// Apply a function to all the coordinates in a geometric object, returning a new object.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::map_coords::MapCoords;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10., 20.);
    /// let p2 = p1.map_coords(|&(x, y)| (x + 1000., y * 2.));
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
    /// let p2: Point<f64> = p1.map_coords(|&(x, y)| (x as f64, y as f64));
    ///
    /// assert_eq!(p2, Point::new(10.0f64, 20.0f64));
    /// ```
    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output
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
    /// use geo::algorithm::map_coords::TryMapCoords;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10., 20.);
    /// let p2 = p1
    ///     .try_map_coords(|&(x, y)| Ok((x + 1000., y * 2.)))
    ///     .unwrap();
    ///
    /// assert_eq!(p2, Point::new(1010., 40.));
    /// ```
    ///
    /// ## Advanced Example: Geometry coordinate conversion using `PROJ`
    ///
    /// ```
    /// // activate the [use-proj] feature in cargo.toml in order to access proj functions
    /// # #[cfg(feature = "use-proj")]
    /// use geo::{Coordinate, Point};
    /// # #[cfg(feature = "use-proj")]
    /// use geo::algorithm::map_coords::TryMapCoords;
    /// # #[cfg(feature = "use-proj")]
    /// use proj::Proj;
    /// // GeoJSON uses the WGS 84 coordinate system
    /// # #[cfg(feature = "use-proj")]
    /// let from = "EPSG:4326";
    /// // The NAD83 / California zone 6 (ftUS) coordinate system
    /// # #[cfg(feature = "use-proj")]
    /// let to = "EPSG:2230";
    /// # #[cfg(feature = "use-proj")]
    /// let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();
    /// # #[cfg(feature = "use-proj")]
    /// let f = |x: f64, y: f64| {
    ///     // proj can accept Point, Coordinate, Tuple, and array values, returning a Result
    ///     let shifted = to_feet.convert((x, y))?;
    ///     Ok((shifted.x(), shifted.y()))
    /// };
    /// # #[cfg(feature = "use-proj")]
    /// // 👽
    /// # #[cfg(feature = "use-proj")]
    /// let usa_m = Point::new(-115.797615, 37.2647978);
    /// # #[cfg(feature = "use-proj")]
    /// let usa_ft = usa_m.try_map_coords(|&(x, y)| f(x, y)).unwrap();
    /// # #[cfg(feature = "use-proj")]
    /// assert_eq!(6693625.67217475, usa_ft.x());
    /// # #[cfg(feature = "use-proj")]
    /// assert_eq!(3497301.5918027186, usa_ft.y());
    /// ```
    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>>
    where
        T: CoordinateType,
        NT: CoordinateType;
}

/// Map a function over all the coordinates in an object in place
pub trait MapCoordsInplace<T> {
    /// Apply a function to all the coordinates in a geometric object, in place
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::map_coords::MapCoordsInplace;
    /// use geo::Point;
    ///
    /// let mut p = Point::new(10., 20.);
    /// p.map_coords_inplace(|&(x, y)| (x + 1000., y * 2.));
    ///
    /// assert_eq!(p, Point::new(1010., 40.));
    /// ```
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy)
    where
        T: CoordinateType;
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Point<T> {
    type Output = Point<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        let new_point = func(&(self.0.x, self.0.y));
        Point::new(new_point.0, new_point.1)
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Point<T> {
    type Output = Point<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>>,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let new_point = func(&(self.0.x, self.0.y))?;
        Ok(Point::new(new_point.0, new_point.1))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Point<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T)) {
        let new_point = func(&(self.0.x, self.0.y));
        self.0.x = new_point.0;
        self.0.y = new_point.1;
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Line<T> {
    type Output = Line<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
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
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(Line::new(
            self.start_point().try_map_coords(func)?.0,
            self.end_point().try_map_coords(func)?.0,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Line<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T)) {
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

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
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
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(LineString::from(
            self.points_iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for LineString<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T)) {
        for p in &mut self.0 {
            let new_coords = func(&(p.x, p.y));
            p.x = new_coords.0;
            p.y = new_coords.1;
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Polygon<T> {
    type Output = Polygon<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
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
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(Polygon::new(
            self.exterior().try_map_coords(func)?,
            self.interiors()
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Polygon<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
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

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        MultiPoint(self.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiPoint<T> {
    type Output = MultiPoint<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(MultiPoint(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiPoint<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for MultiLineString<T> {
    type Output = MultiLineString<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        MultiLineString(self.iter().map(|l| l.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiLineString<T> {
    type Output = MultiLineString<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(MultiLineString(
            self.0
                .iter()
                .map(|l| l.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiLineString<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for MultiPolygon<T> {
    type Output = MultiPolygon<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        MultiPolygon(self.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for MultiPolygon<T> {
    type Output = MultiPolygon<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(MultiPolygon(
            self.0
                .iter()
                .map(|p| p.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for MultiPolygon<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Geometry<T> {
    type Output = Geometry<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        match *self {
            Geometry::Point(ref x) => Geometry::Point(x.map_coords(func)),
            Geometry::Line(ref x) => Geometry::Line(x.map_coords(func)),
            Geometry::LineString(ref x) => Geometry::LineString(x.map_coords(func)),
            Geometry::Polygon(ref x) => Geometry::Polygon(x.map_coords(func)),
            Geometry::MultiPoint(ref x) => Geometry::MultiPoint(x.map_coords(func)),
            Geometry::MultiLineString(ref x) => Geometry::MultiLineString(x.map_coords(func)),
            Geometry::MultiPolygon(ref x) => Geometry::MultiPolygon(x.map_coords(func)),
            Geometry::GeometryCollection(ref x) => Geometry::GeometryCollection(x.map_coords(func)),
            Geometry::Rect(ref x) => Geometry::Rect(x.map_coords(func)),
            Geometry::Triangle(ref x) => Geometry::Triangle(x.map_coords(func)),
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Geometry<T> {
    type Output = Geometry<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
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
            Geometry::Rect(ref x) => Ok(Geometry::Rect(x.try_map_coords(func)?)),
            Geometry::Triangle(ref x) => Ok(Geometry::Triangle(x.try_map_coords(func)?)),
        }
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Geometry<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        match *self {
            Geometry::Point(ref mut x) => x.map_coords_inplace(func),
            Geometry::Line(ref mut x) => x.map_coords_inplace(func),
            Geometry::LineString(ref mut x) => x.map_coords_inplace(func),
            Geometry::Polygon(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiPoint(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiLineString(ref mut x) => x.map_coords_inplace(func),
            Geometry::MultiPolygon(ref mut x) => x.map_coords_inplace(func),
            Geometry::GeometryCollection(ref mut x) => x.map_coords_inplace(func),
            Geometry::Rect(ref mut x) => x.map_coords_inplace(func),
            Geometry::Triangle(ref mut x) => x.map_coords_inplace(func),
        }
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for GeometryCollection<T> {
    type Output = GeometryCollection<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        GeometryCollection(self.iter().map(|g| g.map_coords(func)).collect())
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for GeometryCollection<T> {
    type Output = GeometryCollection<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>> + Copy,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok(GeometryCollection(
            self.0
                .iter()
                .map(|g| g.try_map_coords(func))
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?,
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for GeometryCollection<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        for p in &mut self.0 {
            p.map_coords_inplace(func);
        }
    }
}

fn normalize_rect_bounds<T: PartialOrd>(min: &mut (T, T), max: &mut (T, T)) {
    use std::mem::swap;
    if min.0 > max.0 {
        swap(&mut min.0, &mut max.0);
    }
    if min.1 > max.1 {
        swap(&mut min.1, &mut max.1);
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Rect<T> {
    type Output = Rect<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        let mut new_min = func(&self.min().x_y());
        let mut new_max = func(&self.max().x_y());
        normalize_rect_bounds(&mut new_min, &mut new_max);

        Rect::new(
            Coordinate {
                x: new_min.0,
                y: new_min.1,
            },
            Coordinate {
                x: new_max.0,
                y: new_max.1,
            },
        )
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Rect<T> {
    type Output = Rect<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>>,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let mut new_min = func(&(self.min().x, self.min().y))?;
        let mut new_max = func(&(self.max().x, self.max().y))?;
        normalize_rect_bounds(&mut new_min, &mut new_max);

        Ok(Rect::new(
            Coordinate {
                x: new_min.0,
                y: new_min.1,
            },
            Coordinate {
                x: new_max.0,
                y: new_max.1,
            },
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Rect<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T)) {
        let mut new_min = func(&self.min().x_y());
        let mut new_max = func(&self.max().x_y());

        normalize_rect_bounds(&mut new_min, &mut new_max);
        let mut new_rect = Rect::new(new_min, new_max);

        ::std::mem::swap(self, &mut new_rect);
    }
}

impl<T: CoordinateType, NT: CoordinateType> MapCoords<T, NT> for Triangle<T> {
    type Output = Triangle<NT>;

    fn map_coords(&self, func: impl Fn(&(T, T)) -> (NT, NT) + Copy) -> Self::Output {
        let p1 = func(&self.0.x_y());
        let p2 = func(&self.1.x_y());
        let p3 = func(&self.2.x_y());

        Triangle(
            Coordinate { x: p1.0, y: p1.1 },
            Coordinate { x: p2.0, y: p2.1 },
            Coordinate { x: p3.0, y: p3.1 },
        )
    }
}

impl<T: CoordinateType, NT: CoordinateType> TryMapCoords<T, NT> for Triangle<T> {
    type Output = Triangle<NT>;

    fn try_map_coords(
        &self,
        func: impl Fn(&(T, T)) -> Result<(NT, NT), Box<dyn Error + Send + Sync>>,
    ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let p1 = func(&self.0.x_y())?;
        let p2 = func(&self.1.x_y())?;
        let p3 = func(&self.2.x_y())?;

        Ok(Triangle(
            Coordinate { x: p1.0, y: p1.1 },
            Coordinate { x: p2.0, y: p2.1 },
            Coordinate { x: p3.0, y: p3.1 },
        ))
    }
}

impl<T: CoordinateType> MapCoordsInplace<T> for Triangle<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T)) {
        let p1 = func(&self.0.x_y());
        let p2 = func(&self.1.x_y());
        let p3 = func(&self.2.x_y());

        let mut new_triangle = Triangle(
            Coordinate { x: p1.0, y: p1.1 },
            Coordinate { x: p2.0, y: p2.1 },
            Coordinate { x: p3.0, y: p3.1 },
        );

        ::std::mem::swap(self, &mut new_triangle);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{polygon, Coordinate};

    #[test]
    fn point() {
        let p = Point::new(10., 10.);
        let new_p = p.map_coords(|&(x, y)| (x + 10., y + 100.));
        assert_relative_eq!(new_p.x(), 20.);
        assert_relative_eq!(new_p.y(), 110.);
    }

    #[test]
    fn point_inplace() {
        let mut p2 = Point::new(10f32, 10f32);
        p2.map_coords_inplace(|&(x, y)| (x + 10., y + 100.));
        assert_relative_eq!(p2.x(), 20.);
        assert_relative_eq!(p2.y(), 110.);
    }

    #[test]
    fn rect_inplace() {
        let mut rect = Rect::new((10, 10), (20, 20));
        rect.map_coords_inplace(|&(x, y)| (x + 10, y + 20));
        assert_eq!(rect.min(), Coordinate { x: 20, y: 30 });
        assert_eq!(rect.max(), Coordinate { x: 30, y: 40 });
    }

    #[test]
    fn rect_inplace_normalized() {
        let mut rect = Rect::new((2, 2), (3, 3));
        // Rect's enforce that rect.min is up and left of p2.  Here we test that the points are
        // normalized into a valid rect, regardless of the order they are mapped.
        rect.map_coords_inplace(|&pt| {
            match pt {
                // old min point maps to new max point
                (2, 2) => (4, 4),
                // old max point maps to new min point
                (3, 3) => (1, 1),
                _ => panic!("unexpected point"),
            }
        });

        assert_eq!(rect.min(), Coordinate { x: 1, y: 1 });
        assert_eq!(rect.max(), Coordinate { x: 4, y: 4 });
    }

    #[test]
    fn rect_map_coords() {
        let rect = Rect::new((10, 10), (20, 20));
        let another_rect = rect.map_coords(|&(x, y)| (x + 10, y + 20));
        assert_eq!(another_rect.min(), Coordinate { x: 20, y: 30 });
        assert_eq!(another_rect.max(), Coordinate { x: 30, y: 40 });
    }

    #[test]
    fn rect_try_map_coords() {
        let rect = Rect::new((10, 10), (20, 20));
        let result = rect.try_map_coords(|&(x, y)| Ok((x + 10, y + 20)));
        assert!(result.is_ok());
    }

    #[test]
    fn rect_try_map_coords_normalized() {
        let rect = Rect::new((2, 2), (3, 3));
        // Rect's enforce that rect.min is up and left of p2.  Here we test that the points are
        // normalized into a valid rect, regardless of the order they are mapped.
        let new_rect = rect
            .try_map_coords(|&pt| {
                match pt {
                    // old min point maps to new max point
                    (2, 2) => Ok((4, 4)),
                    // old max point maps to new min point
                    (3, 3) => Ok((1, 1)),
                    _ => panic!("unexpected point"),
                }
            })
            .unwrap();
        assert_eq!(new_rect.min(), Coordinate { x: 1, y: 1 });
        assert_eq!(new_rect.max(), Coordinate { x: 4, y: 4 });
    }

    #[test]
    fn line() {
        let line = Line::from([(0., 0.), (1., 2.)]);
        assert_eq!(
            line.map_coords(|&(x, y)| (x * 2., y)),
            Line::from([(0., 0.), (2., 2.)])
        );
    }

    #[test]
    fn linestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2 = line1.map_coords(|&(x, y)| (x + 10., y - 100.));
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

        let p2 = p.map_coords(|&(x, y)| (x + 10., y - 100.));

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
            mp.map_coords(|&(x, y)| (x + 10., y + 100.)),
            MultiPoint(vec![Point::new(20., 110.), Point::new(10., 0.)])
        );
    }

    #[test]
    fn multilinestring() {
        let line1: LineString<f32> = LineString::from(vec![(0., 0.), (1., 2.)]);
        let line2: LineString<f32> = LineString::from(vec![(-1., 0.), (0., 0.), (1., 2.)]);
        let mline = MultiLineString(vec![line1, line2]);
        let mline2 = mline.map_coords(|&(x, y)| (x + 10., y - 100.));
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

        let mp = MultiPolygon(vec![poly1, poly2]);
        let mp2 = mp.map_coords(|&(x, y)| (x * 2., y + 100.));
        assert_eq!(mp2.0.len(), 2);
        assert_eq!(
            mp2.0[0],
            polygon![
                (x: 0., y: 100.),
                (x: 20., y: 100.),
                (x: 20., y: 110.),
                (x: 0., y: 110.),
                (x: 0., y: 100.),
            ],
        );
        assert_eq!(
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

        let gc = GeometryCollection(vec![p1, line1]);

        assert_eq!(
            gc.map_coords(|&(x, y)| (x + 10., y + 100.)),
            GeometryCollection(vec![
                Geometry::Point(Point::new(20., 110.)),
                Geometry::LineString(LineString::from(vec![(10., 100.), (11., 102.)])),
            ])
        );
    }

    #[test]
    fn convert_type() {
        let p1: Point<f64> = Point::new(1., 2.);
        let p2: Point<f32> = p1.map_coords(|&(x, y)| (x as f32, y as f32));
        assert_relative_eq!(p2.x(), 1f32);
        assert_relative_eq!(p2.y(), 2f32);
    }

    #[cfg(feature = "use-proj")]
    #[test]
    fn test_fallible_proj() {
        use proj::Proj;
        let from = "EPSG:4326";
        let to = "EPSG:2230";
        let to_feet = Proj::new_known_crs(&from, &to, None).unwrap();

        let f = |x: f64, y: f64| {
            let shifted = to_feet.convert((x, y))?;
            Ok((shifted.x(), shifted.y()))
        };
        // 👽
        let usa_m = Point::new(-115.797615, 37.2647978);
        let usa_ft = usa_m.try_map_coords(|&(x, y)| f(x, y)).unwrap();
        assert_eq!(6693625.67217475, usa_ft.x());
        assert_eq!(3497301.5918027186, usa_ft.y());
    }

    #[test]
    fn test_fallible() {
        let f = |x: f64, y: f64| {
            if relative_ne!(x, 2.0) {
                Ok((x * 2., y + 100.))
            } else {
                Err("Ugh".into())
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
        let bad = bad_ls.try_map_coords(|&(x, y)| f(x, y));
        assert!(bad.is_err());
        let good = good_ls.try_map_coords(|&(x, y)| f(x, y));
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

    #[test]
    fn rect_map_invert_coords() {
        let rect = Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. });

        // This call should not panic even though Rect::new
        // constructor panics if min coords > max coords
        rect.map_coords(|&(x, y)| (-x, -y));
    }
}
