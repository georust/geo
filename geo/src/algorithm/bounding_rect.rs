use crate::{CoordNum, GeometryCow, Rect};
pub use geo_types::bounding_rect::*;

impl<T> BoundingRect<T> for GeometryCow<'_, T>
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;
    geo_types::geometry_cow_delegate_impl! {
       fn bounding_rect(&self) -> Self::Output;
    }
}
