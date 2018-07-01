use {Coordinate, CoordinateType};

#[derive(Copy, Clone, Debug)]
pub struct Triangle<T: CoordinateType>(pub Coordinate<T>, pub Coordinate<T>, pub Coordinate<T>);

impl<T: CoordinateType> Triangle<T> {
    pub fn to_array(&self) -> [Coordinate<T>; 3] {
        [self.0, self.1, self.2]
    }
}

impl<IC: Into<Coordinate<T>> + Copy, T: CoordinateType> From<[IC; 3]> for Triangle<T> {
    fn from(array: [IC; 3]) -> Triangle<T> {
        Triangle(array[0].into(), array[1].into(), array[2].into())
    }
}
