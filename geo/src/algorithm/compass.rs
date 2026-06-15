use geo_types::CoordFloat;

use crate::{Coord, CoordNum};
use core::f64::consts::FRAC_1_SQRT_2;

/// Value of `sin(22.5°)`.
const SIN_22_5_DEG: f64 = 0.382_683_432_365_089_8;
/// Value of `cos(22.5°)`.
const COS_22_5_DEG: f64 = 0.923_879_532_511_286_7;

/// One of the four cardinal directions of the compass: north, east, south,
/// and west.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}

/// One of the four ordinal (intercardinal) directions of the compass:
/// northeast, southeast, southwest, and northwest.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum OrdinalDirection {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

/// One of the directions on an eight-point (eight-wise) compass: cardinal and
/// ordinal directions together in one set. Also known as the principal winds.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum EightwiseDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// One of the directions on a sixteen-point (sixteen-wise) compass: eight
/// principal winds together with eight half-winds.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum SixteenwiseDirection {
    North,
    NorthNorthEast,
    NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,
    NorthWest,
    NorthNorthWest,
}

impl CardinalDirection {
    /// Returns the unit vector pointing in this direction, with `x` increasing
    /// towards east and `y` increasing towards north.
    pub fn unit_vector<T: CoordNum>(self) -> Coord<T> {
        match self {
            CardinalDirection::North => Coord {
                x: T::zero(),
                y: T::one(),
            },
            CardinalDirection::East => Coord {
                x: T::one(),
                y: T::zero(),
            },
            CardinalDirection::South => Coord {
                x: T::zero(),
                y: T::zero() - T::one(),
            },
            CardinalDirection::West => Coord {
                x: T::zero() - T::one(),
                y: T::zero(),
            },
        }
    }
}

impl OrdinalDirection {
    /// Returns the unit vector pointing in this direction, with `x` increasing
    /// towards east and `y` increasing towards north.
    pub fn unit_vector<T: CoordFloat>(self) -> Coord<T> {
        match self {
            OrdinalDirection::NorthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
            OrdinalDirection::SouthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            OrdinalDirection::SouthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            OrdinalDirection::NorthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
        }
    }
}

impl EightwiseDirection {
    /// Returns the unit vector pointing in this direction, with `x` increasing
    /// towards east and `y` increasing towards north.
    pub fn unit_vector<T: CoordFloat>(self) -> Coord<T> {
        match self {
            EightwiseDirection::North => Coord {
                x: T::zero(),
                y: T::one(),
            },
            EightwiseDirection::NorthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
            EightwiseDirection::East => Coord {
                x: T::one(),
                y: T::zero(),
            },
            EightwiseDirection::SouthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            EightwiseDirection::South => Coord {
                x: T::zero(),
                y: T::zero() - T::one(),
            },
            EightwiseDirection::SouthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            EightwiseDirection::West => Coord {
                x: T::zero() - T::one(),
                y: T::zero(),
            },
            EightwiseDirection::NorthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
        }
    }
}

impl SixteenwiseDirection {
    /// Returns the unit vector pointing in this direction, with `x` increasing
    /// towards east and `y` increasing towards north.
    pub fn unit_vector<T: CoordFloat>(self) -> Coord<T> {
        match self {
            SixteenwiseDirection::North => Coord {
                x: T::zero(),
                y: T::one(),
            },
            SixteenwiseDirection::NorthNorthEast => Coord {
                x: T::from(SIN_22_5_DEG).unwrap(),
                y: T::from(COS_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::NorthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
            SixteenwiseDirection::EastNorthEast => Coord {
                x: T::from(COS_22_5_DEG).unwrap(),
                y: T::from(SIN_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::East => Coord {
                x: T::one(),
                y: T::zero(),
            },
            SixteenwiseDirection::EastSouthEast => Coord {
                x: T::from(COS_22_5_DEG).unwrap(),
                y: T::from(-SIN_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::SouthEast => Coord {
                x: T::from(FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            SixteenwiseDirection::SouthSouthEast => Coord {
                x: T::from(SIN_22_5_DEG).unwrap(),
                y: T::from(-COS_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::South => Coord {
                x: T::zero(),
                y: T::zero() - T::one(),
            },
            SixteenwiseDirection::SouthSouthWest => Coord {
                x: T::from(-SIN_22_5_DEG).unwrap(),
                y: T::from(-COS_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::SouthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(-FRAC_1_SQRT_2).unwrap(),
            },
            SixteenwiseDirection::WestSouthWest => Coord {
                x: T::from(-COS_22_5_DEG).unwrap(),
                y: T::from(-SIN_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::West => Coord {
                x: T::zero() - T::one(),
                y: T::zero(),
            },
            SixteenwiseDirection::WestNorthWest => Coord {
                x: T::from(-COS_22_5_DEG).unwrap(),
                y: T::from(SIN_22_5_DEG).unwrap(),
            },
            SixteenwiseDirection::NorthWest => Coord {
                x: T::from(-FRAC_1_SQRT_2).unwrap(),
                y: T::from(FRAC_1_SQRT_2).unwrap(),
            },
            SixteenwiseDirection::NorthNorthWest => Coord {
                x: T::from(-SIN_22_5_DEG).unwrap(),
                y: T::from(COS_22_5_DEG).unwrap(),
            },
        }
    }
}

/// Snap [`Coord`], interpreted as a direction vector from the origin, to the
/// nearest direction in one of the sets of compass directions.
///
/// Per the usual mathematical convention, `x` is assumed to increase towards
/// east, `y` to increase towards north.
///
/// Input coordinates are first converted to `f64`.
///
/// # Boundary cases
///
/// A direction vector that points exactly along a sector boundary is
/// equidistant to the two adjacent directions. Such ties are always resolved
/// in favor of the clockwise neighbour, that is, the direction with the greater
/// compass bearing, wrapping back to north at 360 degrees.
///
/// For example, the diagonal `[1, 1]` (half-way between north and east)
/// snaps to [`CardinalDirection::East`], and `[-1, 1]` (bearing 315
/// degrees, half-way between west and north) wraps around to snap to
/// [`CardinalDirection::North`].
///
/// Note that whether a given vector lands exactly on a boundary is subject to
/// floating-point rounding, since the coordinates are first converted to `f64`.
///
/// The zero vector has no well-defined direction; every method returns `None`
/// for it.
pub trait NearestCompassDirection {
    /// Snap to the nearest [`CardinalDirection`], or `None` for zero vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::compass::{CardinalDirection, NearestCompassDirection};
    /// use geo::coord;
    ///
    /// assert_eq!(
    ///     coord! { x: 1.0, y: 3.0 }.nearest_cardinal_direction(),
    ///     Some(CardinalDirection::North)
    /// );
    /// assert_eq!(
    ///     coord! { x: -5, y: -4 }.nearest_cardinal_direction(),
    ///     Some(CardinalDirection::West)
    /// );
    /// assert_eq!(coord! { x: 0, y: 0 }.nearest_cardinal_direction(), None);
    /// ```
    fn nearest_cardinal_direction(&self) -> Option<CardinalDirection>;

    /// Snap to the nearest [`OrdinalDirection`], or `None` for zero vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::compass::{NearestCompassDirection, OrdinalDirection};
    /// use geo::coord;
    ///
    /// assert_eq!(
    ///     coord! { x: 2.0, y: 3.0 }.nearest_ordinal_direction(),
    ///     Some(OrdinalDirection::NorthEast)
    /// );
    /// assert_eq!(
    ///     coord! { x: -3, y: 1 }.nearest_ordinal_direction(),
    ///     Some(OrdinalDirection::NorthWest)
    /// );
    /// assert_eq!(coord! { x: 0, y: 0 }.nearest_ordinal_direction(), None);
    /// ```
    fn nearest_ordinal_direction(&self) -> Option<OrdinalDirection>;

    /// Snap to the nearest [`EightwiseDirection`], or `None` for zero vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::compass::{EightwiseDirection, NearestCompassDirection};
    /// use geo::coord;
    ///
    /// assert_eq!(
    ///     coord! { x: 1.0, y: 1.0 }.nearest_eightwise_direction(),
    ///     Some(EightwiseDirection::NorthEast)
    /// );
    /// assert_eq!(
    ///     coord! { x: 0, y: -2 }.nearest_eightwise_direction(),
    ///     Some(EightwiseDirection::South)
    /// );
    /// assert_eq!(coord! { x: 0, y: 0 }.nearest_eightwise_direction(), None);
    /// ```
    fn nearest_eightwise_direction(&self) -> Option<EightwiseDirection>;

    /// Snap to the nearest [`SixteenwiseDirection`], or `None` for zero vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::compass::{NearestCompassDirection, SixteenwiseDirection};
    /// use geo::coord;
    ///
    /// assert_eq!(
    ///     coord! { x: 1.0, y: 2.0 }.nearest_sixteenwise_direction(),
    ///     Some(SixteenwiseDirection::NorthNorthEast)
    /// );
    /// assert_eq!(
    ///     coord! { x: 0, y: -3 }.nearest_sixteenwise_direction(),
    ///     Some(SixteenwiseDirection::South)
    /// );
    /// assert_eq!(coord! { x: 0, y: 0 }.nearest_sixteenwise_direction(), None);
    /// ```
    fn nearest_sixteenwise_direction(&self) -> Option<SixteenwiseDirection>;
}

impl<T: CoordNum> NearestCompassDirection for Coord<T> {
    fn nearest_cardinal_direction(&self) -> Option<CardinalDirection> {
        Some(match nearest_direction_index(*self, 4)? {
            0 => CardinalDirection::North,
            1 => CardinalDirection::East,
            2 => CardinalDirection::South,
            _ => CardinalDirection::West,
        })
    }

    fn nearest_ordinal_direction(&self) -> Option<OrdinalDirection> {
        Some(
            match (compass_bearing(*self)? / 90.0).floor() as usize % 4 {
                0 => OrdinalDirection::NorthEast,
                1 => OrdinalDirection::SouthEast,
                2 => OrdinalDirection::SouthWest,
                _ => OrdinalDirection::NorthWest,
            },
        )
    }

    fn nearest_eightwise_direction(&self) -> Option<EightwiseDirection> {
        Some(match nearest_direction_index(*self, 8)? {
            0 => EightwiseDirection::North,
            1 => EightwiseDirection::NorthEast,
            2 => EightwiseDirection::East,
            3 => EightwiseDirection::SouthEast,
            4 => EightwiseDirection::South,
            5 => EightwiseDirection::SouthWest,
            6 => EightwiseDirection::West,
            _ => EightwiseDirection::NorthWest,
        })
    }

    fn nearest_sixteenwise_direction(&self) -> Option<SixteenwiseDirection> {
        Some(match nearest_direction_index(*self, 16)? {
            0 => SixteenwiseDirection::North,
            1 => SixteenwiseDirection::NorthNorthEast,
            2 => SixteenwiseDirection::NorthEast,
            3 => SixteenwiseDirection::EastNorthEast,
            4 => SixteenwiseDirection::East,
            5 => SixteenwiseDirection::EastSouthEast,
            6 => SixteenwiseDirection::SouthEast,
            7 => SixteenwiseDirection::SouthSouthEast,
            8 => SixteenwiseDirection::South,
            9 => SixteenwiseDirection::SouthSouthWest,
            10 => SixteenwiseDirection::SouthWest,
            11 => SixteenwiseDirection::WestSouthWest,
            12 => SixteenwiseDirection::West,
            13 => SixteenwiseDirection::WestNorthWest,
            14 => SixteenwiseDirection::NorthWest,
            _ => SixteenwiseDirection::NorthNorthWest,
        })
    }
}

fn compass_bearing<T: CoordNum>(coord: Coord<T>) -> Option<f64> {
    let x = coord.x.to_f64()?;
    let y = coord.y.to_f64()?;

    if x == 0.0 && y == 0.0 {
        return None;
    }

    let degrees = x.atan2(y).to_degrees();

    Some(if degrees < 0.0 {
        degrees + 360.0
    } else {
        degrees
    })
}

fn nearest_direction_index<T: CoordNum>(coord: Coord<T>, count: usize) -> Option<usize> {
    let sector = 360.0 / count as f64;
    let index = (compass_bearing(coord)? / sector).round() as usize;

    Some(index % count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord;

    #[test]
    fn cardinal_snapping() {
        assert_eq!(
            coord! { x: 0.0, y: 1.1 }.nearest_cardinal_direction(),
            Some(CardinalDirection::North)
        );
        assert_eq!(
            coord! { x: 1.2, y: 0.1 }.nearest_cardinal_direction(),
            Some(CardinalDirection::East)
        );

        assert_eq!(
            coord! { x: 0.2, y: -1.3 }.nearest_cardinal_direction(),
            Some(CardinalDirection::South)
        );
        assert_eq!(
            coord! { x: -1.4, y: 0.3 }.nearest_cardinal_direction(),
            Some(CardinalDirection::West)
        );
        assert_eq!(
            coord! { x: 0.4, y: 0.9 }.nearest_cardinal_direction(),
            Some(CardinalDirection::North)
        );
        assert_eq!(
            coord! { x: 1.0, y: 1.0 }.nearest_cardinal_direction(),
            Some(CardinalDirection::East)
        );

        assert_eq!(
            coord! { x: -5i32, y: -4i32 }.nearest_cardinal_direction(),
            Some(CardinalDirection::West)
        );
        assert_eq!(
            coord! { x: 3i64, y: 100i64 }.nearest_cardinal_direction(),
            Some(CardinalDirection::North)
        );
    }

    #[test]
    fn ordinal_snapping() {
        assert_eq!(
            coord! { x: 1.1, y: 1.2 }.nearest_ordinal_direction(),
            Some(OrdinalDirection::NorthEast)
        );
        assert_eq!(
            coord! { x: 1.0, y: -1.3 }.nearest_ordinal_direction(),
            Some(OrdinalDirection::SouthEast)
        );
        assert_eq!(
            coord! { x: -1.1, y: -0.9 }.nearest_ordinal_direction(),
            Some(OrdinalDirection::SouthWest)
        );
        assert_eq!(
            coord! { x: -0.9, y: 0.9 }.nearest_ordinal_direction(),
            Some(OrdinalDirection::NorthWest)
        );
    }

    #[test]
    fn eightwise_snapping() {
        assert_eq!(
            coord! { x: 1.1, y: 1.0 }.nearest_eightwise_direction(),
            Some(EightwiseDirection::NorthEast)
        );
        assert_eq!(
            coord! { x: 0i32, y: -2i32 }.nearest_eightwise_direction(),
            Some(EightwiseDirection::South)
        );
    }

    #[test]
    fn sixteenwise_snapping() {
        assert_eq!(
            coord! { x: 1.0, y: 2.1 }.nearest_sixteenwise_direction(),
            Some(SixteenwiseDirection::NorthNorthEast)
        );
        assert_eq!(
            coord! { x: 2i32, y: -5i32 }.nearest_sixteenwise_direction(),
            Some(SixteenwiseDirection::SouthSouthEast)
        );
        assert_eq!(
            coord! { x: 0i32, y: -3i32 }.nearest_sixteenwise_direction(),
            Some(SixteenwiseDirection::South)
        );
    }

    #[test]
    fn unit_vectors_are_axis_aligned_for_cardinals() {
        assert_eq!(
            CardinalDirection::North.unit_vector(),
            coord! { x: 0.0, y: 1.0 }
        );
        assert_eq!(
            CardinalDirection::East.unit_vector(),
            coord! { x: 1.0, y: 0.0 }
        );
        assert_eq!(
            CardinalDirection::South.unit_vector(),
            coord! { x: 0.0, y: -1.0 }
        );
        assert_eq!(
            CardinalDirection::West.unit_vector(),
            coord! { x: -1.0, y: 0.0 }
        );
    }

    #[test]
    fn unit_vectors_have_unit_magnitude() {
        let sixteen = [
            SixteenwiseDirection::North,
            SixteenwiseDirection::NorthNorthEast,
            SixteenwiseDirection::NorthEast,
            SixteenwiseDirection::EastNorthEast,
            SixteenwiseDirection::East,
            SixteenwiseDirection::EastSouthEast,
            SixteenwiseDirection::SouthEast,
            SixteenwiseDirection::SouthSouthEast,
            SixteenwiseDirection::South,
            SixteenwiseDirection::SouthSouthWest,
            SixteenwiseDirection::SouthWest,
            SixteenwiseDirection::WestSouthWest,
            SixteenwiseDirection::West,
            SixteenwiseDirection::WestNorthWest,
            SixteenwiseDirection::NorthWest,
            SixteenwiseDirection::NorthNorthWest,
        ];
        for direction in sixteen {
            let v = direction.unit_vector::<f64>();
            assert_relative_eq!((v.x * v.x + v.y * v.y).sqrt(), 1.0);
        }
    }

    #[test]
    fn unit_vectors_round_trip_through_snapping() {
        assert_eq!(
            CardinalDirection::West
                .unit_vector::<f64>()
                .nearest_cardinal_direction(),
            Some(CardinalDirection::West)
        );
        assert_eq!(
            OrdinalDirection::SouthEast
                .unit_vector::<f64>()
                .nearest_ordinal_direction(),
            Some(OrdinalDirection::SouthEast)
        );
        assert_eq!(
            EightwiseDirection::NorthWest
                .unit_vector::<f64>()
                .nearest_eightwise_direction(),
            Some(EightwiseDirection::NorthWest)
        );
        assert_eq!(
            SixteenwiseDirection::EastSouthEast
                .unit_vector::<f64>()
                .nearest_sixteenwise_direction(),
            Some(SixteenwiseDirection::EastSouthEast)
        );
    }

    #[test]
    fn zero_vector_is_none() {
        assert_eq!(coord! { x: 0.0, y: 0.0 }.nearest_cardinal_direction(), None);
        assert_eq!(coord! { x: 0.0, y: 0.0 }.nearest_ordinal_direction(), None);
        assert_eq!(
            coord! { x: 0i32, y: 0i32 }.nearest_eightwise_direction(),
            None
        );
        assert_eq!(
            coord! { x: 0i32, y: 0i32 }.nearest_sixteenwise_direction(),
            None
        );
    }
}
