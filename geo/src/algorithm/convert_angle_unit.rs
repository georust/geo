use geo_types::Coord;
use geo_types::CoordFloat;

use crate::{MapCoords, MapCoordsInPlace};

pub trait ToRadians<T: CoordFloat>:
    Sized + MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>
{
    fn to_radians(&self) -> Self {
        self.map_coords(|Coord { x, y }| Coord {
            x: x.to_radians(),
            y: y.to_radians(),
        })
    }

    fn to_radians_in_place(&mut self) {
        self.map_coords_in_place(|Coord { x, y }| Coord {
            x: x.to_radians(),
            y: y.to_radians(),
        })
    }
}
impl<T: CoordFloat, G: MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>> ToRadians<T> for G {}

pub trait ToDegrees<T: CoordFloat>:
    Sized + MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>
{
    fn to_degrees(&self) -> Self {
        self.map_coords(|Coord { x, y }| Coord {
            x: x.to_degrees(),
            y: y.to_degrees(),
        })
    }

    fn to_degrees_in_place(&mut self) {
        self.map_coords_in_place(|Coord { x, y }| Coord {
            x: x.to_degrees(),
            y: y.to_degrees(),
        })
    }
}
impl<T: CoordFloat, G: MapCoords<T, T, Output = Self> + MapCoordsInPlace<T>> ToDegrees<T> for G {}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use approx::assert_relative_eq;
    use geo_types::Line;

    use super::*;

    fn line_degrees_mock() -> Line {
        Line::new((90.0, 180.), (0., -90.))
    }

    fn line_radians_mock() -> Line {
        Line::new((PI / 2., PI), (0., -PI / 2.))
    }

    #[test]
    fn converts_to_radians() {
        assert_relative_eq!(line_radians_mock(), line_degrees_mock().to_radians())
    }

    #[test]
    fn converts_to_radians_in_place() {
        let mut line = line_degrees_mock();
        line.to_radians_in_place();
        assert_relative_eq!(line_radians_mock(), line)
    }

    #[test]
    fn converts_to_degrees() {
        assert_relative_eq!(line_degrees_mock(), line_radians_mock().to_degrees())
    }

    #[test]
    fn converts_to_degrees_in_place() {
        let mut line = line_radians_mock();
        line.to_degrees_in_place();
        assert_relative_eq!(line_degrees_mock(), line)
    }
}
