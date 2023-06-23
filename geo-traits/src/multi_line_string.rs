use super::line_string::LineStringTrait;
use geo_types::{CoordNum, LineString, MultiLineString};
use std::iter::Cloned;
use std::ops::SubAssign;
use std::slice::Iter;

pub trait MultiLineStringTrait<'a>: Send + Sync {
    type ItemType: 'a + LineStringTrait<'a>;
    type Iter: ExactSizeIterator<Item = Self::ItemType>;

    /// An iterator over the LineStrings in this MultiLineString
    fn lines(&'a self) -> Self::Iter;

    /// The number of lines in this MultiLineString
    fn num_lines(&'a self) -> usize;

    /// Access to a specified line in this MultiLineString
    /// Will return None if the provided index is out of bounds
    fn line(&'a self, i: usize) -> Option<Self::ItemType>;
}

impl<'a, T: CoordNum + Send + Sync + SubAssign + 'a> MultiLineStringTrait<'a>
    for MultiLineString<T>
{
    type ItemType = LineString<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn lines(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_lines(&'a self) -> usize {
        self.0.len()
    }

    fn line(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}

impl<'a, T: CoordNum + Send + Sync + SubAssign + 'a> MultiLineStringTrait<'a>
    for &MultiLineString<T>
{
    type ItemType = LineString<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn lines(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_lines(&'a self) -> usize {
        self.0.len()
    }

    fn line(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}
