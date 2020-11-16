use num_traits::Float;
use std::iter;
use crate::{Coordinate, Point, Polygon, LineString, Line, MultiPoint, MultiPolygon,
            MultiLineString};

// TODO add geometry impl
// TODO move away from Float

pub trait CoordsIter<'a, T: Float + 'a> {
    type Iter: Iterator<Item = &'a Coordinate<T>>;

    fn coords_iter(&'a self) -> Self::Iter;
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for Point<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(iter::once(&self.0))
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for Line<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(iter::once(&self.start).chain(iter::once(&self.end)))
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for LineString<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter())
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for Polygon<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.exterior().coords_iter().chain(
            self.interiors().iter().flat_map(
                |i| {
                    i.coords_iter()
                },
            ),
        ))
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for MultiPoint<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for MultiLineString<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

impl<'a, T: Float + 'a> CoordsIter<'a, T> for MultiPolygon<T> {
    type Iter = Box<dyn Iterator<Item = &'a Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}
