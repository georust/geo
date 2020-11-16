use num_traits::Float;
use std::iter;
use crate::{Coordinate, Point, Polygon, LineString, Line, MultiPoint, MultiPolygon,
            MultiLineString};

pub trait CoordsIter<T: Float> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a>;
}

impl<T: Float> CoordsIter<T> for Point<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(iter::once(&self.0))
    }
}

impl<T: Float> CoordsIter<T> for Line<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(iter::once(&self.start).chain(iter::once(&self.end)))
    }
}

impl<T: Float> CoordsIter<T> for LineString<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter())
    }
}

impl<T: Float> CoordsIter<T> for Polygon<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.exterior().coords_iter().chain(
            self.interiors().iter().flat_map(
                |i| {
                    i.coords_iter()
                },
            ),
        ))
    }
}

impl<T: Float> CoordsIter<T> for MultiPoint<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

impl<T: Float> CoordsIter<T> for MultiLineString<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

impl<T: Float> CoordsIter<T> for MultiPolygon<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}
