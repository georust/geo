use super::{Intersects, value_in_range};
use crate::*;

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn intersects(&self, other: &Rect<T>) -> bool {
        let x_overlap = value_in_range(self.min().x, other.min().x, other.max().x)
            || value_in_range(other.min().x, self.min().x, self.max().x);

        let y_overlap = value_in_range(self.min().y, other.min().y, other.max().y)
            || value_in_range(other.min().y, self.min().y, self.max().y);

        x_overlap && y_overlap
    }
}
