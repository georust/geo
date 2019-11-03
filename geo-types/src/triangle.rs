use crate::{Coordinate, CoordinateType, Line};
use std::hash::{Hash, Hasher};

/// A bounded 2D area whose three vertices are defined by `Coordinate`s.
#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle<T: CoordinateType>(pub Coordinate<T>, pub Coordinate<T>, pub Coordinate<T>);

impl<T: CoordinateType> Triangle<T> {
    pub fn to_array(&self) -> [Coordinate<T>; 3] {
        [self.0, self.1, self.2]
    }

    pub fn to_lines(&self) -> [Line<T>; 3] {
        [
            Line::new(self.0, self.1),
            Line::new(self.1, self.2),
            Line::new(self.2, self.0),
        ]
    }
}

impl<IC: Into<Coordinate<T>> + Copy, T: CoordinateType> From<[IC; 3]> for Triangle<T> {
    fn from(array: [IC; 3]) -> Triangle<T> {
        Triangle(array[0].into(), array[1].into(), array[2].into())
    }
}

impl<T: CoordinateType + Hash> Hash for Triangle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}
