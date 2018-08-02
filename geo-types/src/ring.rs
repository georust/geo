use {Coordinate, CoordinateType, Line, Point};
use std::ops::Index;

#[derive(PartialEq, Clone, Debug)]
pub struct Ring<T>(Vec<Coordinate<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType> Ring<T> {
    /// The resulting `Ring` will be implicitly closed.
    ///
    /// # Panics
    ///
    /// This method panics if `coords` has fewer than three coordinates.
    pub fn from_coordinates(mut coords: Vec<Coordinate<T>>) -> Result<Self, ()> {
        let len = coords.len();
        if len < 3 {
            return Err(());
        }
        // unsafe: we just checked the bounds above, no need to check again
        let (first, last) = unsafe {
            (*coords.get_unchecked(0), *coords.get_unchecked(len))
        };
        if first != last {
            coords.push(first);
        }
        Ok(Ring(coords))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn coordinates<'a>(&'a self) -> impl Iterator<Item = Coordinate<T>> + 'a {
        self.0.iter().map(|c| *c)
    }

    pub fn lines<'a>(&'a self) -> impl ExactSizeIterator + Iterator<Item = Line<T>> + 'a {
        self.0.windows(2).map(|w| {
            // unsafe: slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    pub fn points_iter(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }
}

impl<T: CoordinateType> Index<usize> for Ring<T> {
    type Output = Coordinate<T>;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

pub struct PointsIter<'a, T: CoordinateType + 'a>(::std::slice::Iter<'a, Coordinate<T>>);

impl<'a, T: CoordinateType> Iterator for PointsIter<'a, T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point(*c))
    }
}

impl<'a, T: CoordinateType> DoubleEndedIterator for PointsIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point(*c))
    }
}
