use crate::{AffineTransform, CoordNum, MapCoords, MapCoordsInPlace, Point};

/// An affine transformation which scales a geometry up or down by a factor.
///
/// The point of origin is *usually* given as the 2D bounding box centre of the geometry, but
/// any coordinate may be specified.
///
/// ## Performance
///
/// If you will be performing multiple transformations, like [`Scale`](crate::Scale),
/// [`Skew`](crate::Skew), [`Translate`](crate::Translate), or [`Rotate`](crate::Rotate), it is more
/// efficient to compose the transformations and apply them as a single operation using the
/// [`AffineOps`](crate::AffineOps) trait.
///
/// # Examples
///
/// ```
/// use geo::Scale;
/// use geo::{line_string, BoundingRect, LineString};
///
/// let ls: LineString<f64> = line_string![
///     (x: 0.0f64, y: 0.0f64),
///     (x: 10.0f64, y: 10.0f64),
/// ];
/// let origin = ls.bounding_rect().unwrap().center();
/// let scaled = ls.scale(10.0, 10.0, origin.into());
///
/// assert_eq!(scaled, line_string![
///     (x: -45.0f64, y: -45.0f64),
///     (x: 55.0f64, y: 55.0f64)
/// ]);
/// ```
///
pub trait Scale<T> {
    fn scale(&self, x: T, y: T, origin: Point<T>) -> Self
    where
        T: CoordNum;
}

impl<T, G> Scale<T> for G
where
    T: CoordNum,
    G: MapCoords<T, T, Output = G> + MapCoordsInPlace<T>,
{
    fn scale(&self, x: T, y: T, origin: Point<T>) -> Self {
        let affineop = AffineTransform::scale(x, y, origin);
        self.map_coords(|coord| affineop.apply(coord))
    }
}
