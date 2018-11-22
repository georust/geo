use num_traits::Float;
use prelude::*;
use {Coordinate, CoordinateType, Line};

#[derive(PartialEq, Clone, Debug)]
pub struct Ring<T>(Vec<Coordinate<T>>)
where
    T: CoordinateType;

impl<T> Ring<T>
where
    T: CoordinateType,
{
    /// The resulting `Ring` will be implicitly closed.
    ///
    /// # Panics
    ///
    /// This method panics if `coords` has fewer than three coordinates.
    pub fn new(mut coords: Vec<Coordinate<T>>) -> Self {
        if coords.len() < 3 {
            panic!("Unable to create ring with {} coordinates", coords.len());
        }
        if coords[0] != coords[coords.len() - 1] {
            let last = coords[0].clone();
            coords.push(last);
        }
        Ring(coords)
    }

    pub fn lines<'a>(&'a self) -> impl ExactSizeIterator + Iterator<Item = Line<T>> + 'a {
        self.0.windows(2).map(|w| Line::new(w[0], w[1]))
    }
}

impl<T> Area<T> for Ring<T>
where
    T: CoordinateType + Float,
{
    fn area(&self) -> T {
        let two = T::one() + T::one();
        self.lines()
            .map(|l| l.determinant())
            .fold(T::zero(), |sum, d| sum + d)
            / two
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Coordinate;

    #[test]
    fn test_ring_triangle_area() {
        let ring = Ring(vec![
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 10., y: 0. },
            Coordinate { x: 0., y: 10. },
        ]);
        assert_eq!(50., ring.area());
    }

}
