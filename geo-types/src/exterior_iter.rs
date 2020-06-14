use std::{iter, slice};

pub trait ExteriorCoordsIterator {
    type T: crate::CoordinateType;
    type Iter: Iterator<Item = crate::Coordinate<Self::T>>;

    fn exterior_coords_iter(self) -> Self::Iter;
}

impl<'a, T: 'a + crate::CoordinateType> ExteriorCoordsIterator for &'a crate::Polygon<T> {
    type T = T;
    type Iter = iter::Copied<slice::Iter<'a, crate::Coordinate<T>>>;

    fn exterior_coords_iter(self) -> Self::Iter {
        self.exterior().0.iter().copied()
    }
}

pub trait BoundingRect<T: crate::CoordinateType> {
    type Output;

    fn bounding_rect(&self) -> Self::Output;
}

impl<T, I> BoundingRect<T> for I
where
    I: ExteriorCoordsIterator,
    T: crate::CoordinateType,
{
    type Output = crate::Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        unimplemented!()
    }
}

// impl<'a, T: crate::CoordinateType, I> rstar::RTreeObject for crate::Poly
// where
//     I: ExteriorCoordsIter<T = T>,
// {
//     type Envelope = rstar::AABB<crate::Point<T>>;

//     fn envelope(self) -> Self::Envelope {
//         // rstar::AABB::from_points(self.exterior_coords_iter())
//         unimplemented!()
//     }
// }
