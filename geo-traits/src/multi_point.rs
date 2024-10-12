use crate::{Dimension, MultiPointIterator, PointTrait};
use geo_types::{CoordNum, MultiPoint, Point};

/// A trait for accessing data from a generic MultiPoint.
pub trait MultiPointTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying Point, which implements [PointTrait]
    type ItemType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// An iterator over the points in this MultiPoint
    fn points(&self) -> impl Iterator<Item = Self::ItemType<'_>> {
        MultiPointIterator::new(self, 0, self.num_points())
    }

    /// The number of points in this MultiPoint
    fn num_points(&self) -> usize;

    /// Access to a specified point in this MultiPoint
    /// Will return None if the provided index is out of bounds
    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_points() {
            None
        } else {
            unsafe { Some(self.point_unchecked(i)) }
        }
    }

    /// Access to a specified point in this MultiPoint
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn point_unchecked(&self, i: usize) -> Self::ItemType<'_>;
}

impl<T: CoordNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type ItemType<'a> = &'a Point<Self::T> where Self: 'a;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> MultiPointTrait for &'a MultiPoint<T> {
    type T = T;
    type ItemType<'b> = &'a Point<Self::T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tmp() {
        let mp = MultiPoint::new(vec![
            Point::new(0.0, 1.0),
            Point::new(2.0, 3.0),
            Point::new(4.0, 5.0),
        ]);
        MultiPointTrait::points(&mp).for_each(|p| {
            dbg!(p);
        });
    }
}
