use num_traits::Float;
use std::iter;
use types::{Coordinate, Point, Polygon, LineString, Line, MultiPoint, MultiPolygon,
            MultiLineString};

pub trait CoordsIter<T: Float> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a>;
    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a>;
}

impl<T: Float> CoordsIter<T> for Point<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(iter::once(&self.0))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(iter::once(&mut self.0))
    }
}

impl<T: Float> CoordsIter<T> for Line<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(iter::once(&self.start.0).chain(iter::once(&self.end.0)))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(iter::once(&mut self.start.0).chain(
            iter::once(&mut self.end.0),
        ))
    }
}

impl<T: Float> CoordsIter<T> for LineString<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().map(|n| &n.0))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(self.0.iter_mut().map(|n| &mut n.0))
    }
}

impl<T: Float> CoordsIter<T> for Polygon<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.exterior.coords_iter().chain(
            self.interiors.iter().flat_map(
                |i| {
                    i.coords_iter()
                },
            ),
        ))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(self.exterior.coords_iter_mut().chain(
            self.interiors.iter_mut().flat_map(|i| i.coords_iter_mut()),
        ))
    }
}

impl<T: Float> CoordsIter<T> for MultiPoint<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(self.0.iter_mut().flat_map(|m| m.coords_iter_mut()))
    }
}

impl<T: Float> CoordsIter<T> for MultiLineString<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(self.0.iter_mut().flat_map(|m| m.coords_iter_mut()))
    }
}

impl<T: Float> CoordsIter<T> for MultiPolygon<T> {
    fn coords_iter<'a>(&'a self) -> Box<Iterator<Item = &'a Coordinate<T>> + 'a> {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }

    fn coords_iter_mut<'a>(&'a mut self) -> Box<Iterator<Item = &'a mut Coordinate<T>> + 'a> {
        Box::new(self.0.iter_mut().flat_map(|m| m.coords_iter_mut()))
    }
}
