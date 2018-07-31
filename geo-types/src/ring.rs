use {Coordinate, CoordinateType};
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

    pub fn coordinates(&self) -> impl Iterator<Item = &Coordinate<T>> {
        self.0.iter()
    }
}

impl<T: CoordinateType> Index<usize> for Ring<T> {
    type Output = Coordinate<T>;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}
