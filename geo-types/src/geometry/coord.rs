use crate::{coord, CoordNum, Point};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use serde::{Deserialize, Serialize};

/// A lightweight struct used to store coordinates on the 2-dimensional
/// Cartesian plane.
///
/// Unlike `Point` (which in the future may contain additional information such
/// as an envelope, a precision model, and spatial reference system
/// information), a `Coord` only contains ordinate values and accessor
/// methods.
///T
/// This type implements the [vector space] operations:
/// [`Add`], [`Sub`], [`Neg`], [`Zero`],
/// [`Mul<T>`][`Mul`], and [`Div<T>`][`Div`] traits.
///
/// # Semantics
///
/// This type does not represent any geospatial primitive,
/// but is used in their definitions. The only requirement
/// is that the coordinates it contains are valid numbers
/// (for eg. not `f64::NAN`).
///
/// [vector space]: //en.wikipedia.org/wiki/Vector_space
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
//#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord<const N: usize, T: CoordNum = f64> {
        pub coords: [T; N],
}

impl<
                T: Debug
                        + num_traits::Num
                        + std::marker::Copy
                        + num_traits::NumCast
                        + PartialOrd
                        + Default,
                const N: usize,
        > Default for Coord<N, T>
{
        fn default() -> Self {
                Coord {
                        coords: [Default::default(); N],
                }
        }
}

#[deprecated(note = "Renamed to `geo_types::Coord` (or `geo::Coord`)")]
pub type Coordinate<const N: usize, T = f64> = Coord<N, T>;

impl<T: CoordNum> From<(T, T)> for Coord<2, T> {
        #[inline]
        fn from(coords: (T, T)) -> Self {
                Coord {
                        coords: [coords.0, coords.1],
                }
        }
}

impl<T: CoordNum> From<[T; 2]> for Coord<2, T> {
        #[inline]
        fn from(coords: [T; 2]) -> Self {
                Coord {
                        coords: coords.into(),
                }
        }
}

impl<T: CoordNum, const N: usize> From<Point<T>> for Coord<N, T> {
        #[inline]
        fn from(point: Point<T>) -> Self {
                point.0
        }
}

impl<T: CoordNum> From<Coord<2, T>> for (T, T) {
        #[inline]
        fn from(coord: Coord<2, T>) -> Self {
                (coord.coords[0], coord.coords[1])
        }
}

impl<T: CoordNum> From<Coord<2, T>> for [T; 2] {
        #[inline]
        fn from(coord: Coord<2, T>) -> Self {
                [coord.coords[0], coord.coords[1]]
        }
}

impl<T: CoordNum> Coord<2, T> {
        /// Returns a tuple that contains the x/horizontal & y/vertical component of the coordinate.
        ///
        /// # Examples
        ///
        /// ```
        /// use geo_types::coord;
        ///
        /// let c = coord! {
        ///     x: 40.02f64,
        ///     y: 116.34,
        /// };
        /// let (x, y) = c.x_y();
        ///
        /// assert_eq!(y, 116.34);
        /// assert_eq!(x, 40.02f64);
        /// ```
        #[inline]
        pub fn x_y(&self) -> (T, T) {
                (self.coords[0], self.coords[1])
        }
}

use core::{
        fmt::Debug,
        ops::{Add, Div, Mul, Neg, Sub},
};

/// Negate a coordinate.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = -p;
///
/// assert_eq!(q.x, -p.x);
/// assert_eq!(q.y, -p.y);
/// ```
impl<T, const N: usize> Neg for Coord<N, T>
where
        T: CoordNum + Neg<Output = T>,
{
        type Output = Self;

        #[inline]
        fn neg(self) -> Self {
                Coord {
                        coords: self.coords.map(|x| x.neg()).into(),
                }
        }
}

/// Add two coordinates.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = coord! { x: 1.5, y: 2.5 };
/// let sum = p + q;
///
/// assert_eq!(sum.x, 2.75);
/// assert_eq!(sum.y, 5.0);
/// ```
impl<T: CoordNum + Default, const N: usize> Add for Coord<N, T> {
        type Output = Self;

        #[inline]
        fn add(self, rhs: Self) -> Self {
                let mut coords = [Default::default(); N];
                for i in 0..N {
                        coords[i] = self.coords[i] + rhs.coords[i];
                }
                Coord { coords }
        }
}

/// Subtract a coordinate from another.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.5, y: 2.5 };
/// let q = coord! { x: 1.25, y: 2.5 };
/// let diff = p - q;
///
/// assert_eq!(diff.x, 0.25);
/// assert_eq!(diff.y, 0.);
/// ```
impl<T: CoordNum + std::default::Default, const N: usize> Sub for Coord<N, T> {
        type Output = Self;

        #[inline]
        fn sub(self, rhs: Self) -> Self {
                let mut coords = [Default::default(); N];
                for i in 0..N {
                        coords[i] = self.coords[i] - rhs.coords[i];
                }
                Coord { coords }
        }
}

/// Multiply coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = p * 4.;
///
/// assert_eq!(q.x, 5.0);
/// assert_eq!(q.y, 10.0);
/// ```
impl<T: CoordNum + Default, const N: usize> Mul<T> for Coord<N, T> {
        type Output = Self;

        #[inline]
        fn mul(self, rhs: T) -> Self {
                let mut coords = [Default::default(); N];
                for i in 0..N {
                        coords[i] = self.coords[i] * rhs;
                }
                Coord { coords }
        }
}

/// Divide coordinate wise by a scalar.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 5., y: 10. };
/// let q = p / 4.;
///
/// assert_eq!(q.x, 1.25);
/// assert_eq!(q.y, 2.5);
/// ```
impl<T: CoordNum + Default, const N: usize> Div<T> for Coord<N, T> {
        type Output = Self;

        #[inline]
        fn div(self, rhs: T) -> Self {
                let mut coords = [Default::default(); N];
                for i in 0..N {
                        coords[i] = self.coords[i] / rhs;
                }
                Coord { coords }
        }
}

use num_traits::Zero;
/// Create a coordinate at the origin.
///
/// # Examples
///
/// ```
/// use geo_types::Coord;
/// use num_traits::Zero;
///
/// let p: Coord = Zero::zero();
///
/// assert_eq!(p.x, 0.);
/// assert_eq!(p.y, 0.);
/// ```
impl<T: CoordNum, const N: usize> Coord<N, T> {
        #[inline]
        pub fn zero() -> Self {
                Coord {
                        coords: [T::zero(); N],
                }
        }
}

impl<T: CoordNum + Default, const N: usize> Zero for Coord<N, T> {
        #[inline]
        fn zero() -> Self {
                Self::zero()
        }
        #[inline]
        fn is_zero(&self) -> bool {
                if N == 0 {
                        return false;
                }
                let mut ret = true;
                for coord in &self.coords {
                        ret &= coord.is_zero();
                }
                ret
        }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + AbsDiffEq, const N: usize> AbsDiffEq for Coord<N, T>
where
        T::Epsilon: Copy,
{
        type Epsilon = T::Epsilon;

        #[inline]
        fn default_epsilon() -> T::Epsilon {
                T::default_epsilon()
        }

        #[inline]
        fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
                if N == 0 {
                        return false;
                }
                let mut ret = true;
                for i in 0..N {
                        ret &= T::abs_diff_eq(&self.coords[i], &other.coords[i], epsilon)
                }
                ret
        }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + RelativeEq, const N: usize> RelativeEq for Coord<N, T>
where
        T::Epsilon: Copy,
{
        #[inline]
        fn default_max_relative() -> T::Epsilon {
                T::default_max_relative()
        }

        #[inline]
        fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
                if N == 0 {
                        return false;
                }
                let mut ret = true;
                for i in 0..N {
                        ret &= T::relative_eq(
                                &self.coords[i],
                                &other.coords[i],
                                epsilon,
                                max_relative,
                        )
                }
                ret
        }
}

#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + UlpsEq, const N: usize> UlpsEq for Coord<N, T>
where
        T::Epsilon: Copy,
{
        #[inline]
        fn default_max_ulps() -> u32 {
                T::default_max_ulps()
        }

        #[inline]
        fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
                if N == 0 {
                        return false;
                }
                let mut ret = true;
                for i in 0..N {
                        ret &= T::ulps_eq(&self.coords[i], &other.coords[i], epsilon, max_ulps)
                }
                ret
        }
}

#[cfg(feature = "rstar_0_8")]
impl<T> ::rstar_0_8::Point for Coord<T>
where
        T: ::num_traits::Float + ::rstar_0_8::RTreeNum,
{
        type Scalar = T;

        const DIMENSIONS: usize = 2;

        #[inline]
        fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
                coord! {
                    x: generator(0),
                    y: generator(1),
                }
        }

        #[inline]
        fn nth(&self, index: usize) -> Self::Scalar {
                match index {
                        0 => self.x,
                        1 => self.y,
                        _ => unreachable!(),
                }
        }

        #[inline]
        fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
                match index {
                        0 => &mut self.x,
                        1 => &mut self.y,
                        _ => unreachable!(),
                }
        }
}

#[cfg(feature = "rstar_0_9")]
impl<T> ::rstar_0_9::Point for Coord<T>
where
        T: ::num_traits::Float + ::rstar_0_9::RTreeNum,
{
        type Scalar = T;

        const DIMENSIONS: usize = 2;

        #[inline]
        fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
                coord! {
                    x: generator(0),
                    y: generator(1),
                }
        }

        #[inline]
        fn nth(&self, index: usize) -> Self::Scalar {
                match index {
                        0 => self.x,
                        1 => self.y,
                        _ => unreachable!(),
                }
        }

        #[inline]
        fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
                match index {
                        0 => &mut self.x,
                        1 => &mut self.y,
                        _ => unreachable!(),
                }
        }
}

#[cfg(feature = "rstar_0_10")]
impl<T> ::rstar_0_10::Point for Coord<T>
where
        T: ::num_traits::Float + ::rstar_0_10::RTreeNum,
{
        type Scalar = T;

        const DIMENSIONS: usize = 2;

        #[inline]
        fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
                coord! {
                    x: generator(0),
                    y: generator(1),
                }
        }

        #[inline]
        fn nth(&self, index: usize) -> Self::Scalar {
                match index {
                        0 => self.x,
                        1 => self.y,
                        _ => unreachable!(),
                }
        }

        #[inline]
        fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
                match index {
                        0 => &mut self.x,
                        1 => &mut self.y,
                        _ => unreachable!(),
                }
        }
}

#[cfg(feature = "rstar_0_11")]
impl<T> ::rstar_0_11::Point for Coord<T>
where
        T: ::num_traits::Float + ::rstar_0_11::RTreeNum,
{
        type Scalar = T;

        const DIMENSIONS: usize = 2;

        #[inline]
        fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
                coord! {
                    x: generator(0),
                    y: generator(1),
                }
        }

        #[inline]
        fn nth(&self, index: usize) -> Self::Scalar {
                match index {
                        0 => self.x,
                        1 => self.y,
                        _ => unreachable!(),
                }
        }

        #[inline]
        fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
                match index {
                        0 => &mut self.x,
                        1 => &mut self.y,
                        _ => unreachable!(),
                }
        }
}

#[cfg(feature = "rstar_0_12")]
impl<T: Default, const N: usize> ::rstar_0_12::Point for Coord<N, T>
where
        T: ::num_traits::Float + ::rstar_0_12::RTreeNum,
{
        type Scalar = T;

        const DIMENSIONS: usize = N;

        #[inline]
        fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
                let mut coords = [T::default(); N];
                for i in 0..N {
                        coords[i] = generator(i);
                }
                Coord { coords }
        }

        #[inline]
        fn nth(&self, index: usize) -> Self::Scalar {
                if index >= N {
                        unreachable!()
                }
                self.coords[index]
        }

        #[inline]
        fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
                if index >= N {
                        unreachable!()
                }
                &mut self.coords[index]
        }
}
