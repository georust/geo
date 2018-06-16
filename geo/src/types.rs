use std::ops::Add;
use std::ops::AddAssign;

use num_traits::{Float, ToPrimitive};
use ::{CoordinateType, Point};

pub static COORD_PRECISION: f32 = 1e-1; // 0.1m

/// A container for the bounding box of a [`Geometry`](enum.Geometry.html)
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Bbox<T>
where
    T: CoordinateType,
{
    pub xmin: T,
    pub xmax: T,
    pub ymin: T,
    pub ymax: T,
}

/// A container for indices of the minimum and maximum points of a [`Geometry`](enum.Geometry.html)
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Extremes {
    pub ymin: usize,
    pub xmax: usize,
    pub ymax: usize,
    pub xmin: usize,
}

impl From<Vec<usize>> for Extremes {
    fn from(original: Vec<usize>) -> Extremes {
        Extremes {
            ymin: original[0],
            xmax: original[1],
            ymax: original[2],
            xmin: original[3],
        }
    }
}

/// A container for the coordinates of the minimum and maximum points of a [`Geometry`](enum.Geometry.html)
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ExtremePoint<T>
where
    T: CoordinateType,
{
    pub ymin: Point<T>,
    pub xmax: Point<T>,
    pub ymax: Point<T>,
    pub xmin: Point<T>,
}

impl<T> Add for Bbox<T>
where
    T: CoordinateType + ToPrimitive,
{
    type Output = Bbox<T>;

    /// Add a BoundingBox to the given BoundingBox.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Bbox;
    ///
    /// let bbox0 = Bbox{xmin: 0.,  xmax: 10000., ymin: 10., ymax: 100.};
    /// let bbox1 = Bbox{xmin: 100., xmax: 1000.,  ymin: 100.,  ymax: 1000.};
    /// let bbox = bbox0 + bbox1;
    ///
    /// assert_eq!(0., bbox.xmin);
    /// assert_eq!(10000., bbox.xmax);
    /// assert_eq!(10., bbox.ymin);
    /// assert_eq!(1000., bbox.ymax);
    /// ```
    fn add(self, rhs: Bbox<T>) -> Bbox<T> {
        Bbox {
            xmin: if self.xmin <= rhs.xmin {
                self.xmin
            } else {
                rhs.xmin
            },
            xmax: if self.xmax >= rhs.xmax {
                self.xmax
            } else {
                rhs.xmax
            },
            ymin: if self.ymin <= rhs.ymin {
                self.ymin
            } else {
                rhs.ymin
            },
            ymax: if self.ymax >= rhs.ymax {
                self.ymax
            } else {
                rhs.ymax
            },
        }
    }
}

impl<T> AddAssign for Bbox<T>
where
    T: CoordinateType + ToPrimitive,
{
    /// Add a BoundingBox to the given BoundingBox.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Bbox;
    ///
    /// let mut bbox0 = Bbox{xmin: 0.,  xmax: 10000., ymin: 10., ymax: 100.};
    /// let bbox1 = Bbox{xmin: 100., xmax: 1000.,  ymin: 100.,  ymax: 1000.};
    /// bbox0 += bbox1;
    ///
    /// assert_eq!(0., bbox0.xmin);
    /// assert_eq!(10000., bbox0.xmax);
    /// assert_eq!(10., bbox0.ymin);
    /// assert_eq!(1000., bbox0.ymax);
    /// ```
    fn add_assign(&mut self, rhs: Bbox<T>) {
        self.xmin = if self.xmin <= rhs.xmin {
            self.xmin
        } else {
            rhs.xmin
        };
        self.xmax = if self.xmax >= rhs.xmax {
            self.xmax
        } else {
            rhs.xmax
        };
        self.ymin = if self.ymin <= rhs.ymin {
            self.ymin
        } else {
            rhs.ymin
        };
        self.ymax = if self.ymax >= rhs.ymax {
            self.ymax
        } else {
            rhs.ymax
        };
    }
}

/// The result of trying to find the closest spot on an object to a point.
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Closest<F: Float> {
    /// The point actually intersects with the object.
    Intersection(Point<F>),
    /// There is exactly one place on this object which is closest to the point.
    SinglePoint(Point<F>),
    /// There are two or more (possibly infinite or undefined) possible points.
    Indeterminate,
}

impl<F: Float> Closest<F> {
    /// Compare two `Closest`s relative to `p` and return a copy of the best
    /// one.
    pub fn best_of_two(&self, other: &Self, p: Point<F>) -> Self {
        use algorithm::euclidean_distance::EuclideanDistance;

        let left = match *self {
            Closest::Indeterminate => return *other,
            Closest::Intersection(_) => return *self,
            Closest::SinglePoint(l) => l,
        };
        let right = match *other {
            Closest::Indeterminate => return *self,
            Closest::Intersection(_) => return *other,
            Closest::SinglePoint(r) => r,
        };

        if left.euclidean_distance(&p) <= right.euclidean_distance(&p) {
            *self
        } else {
            *other
        }
    }
}
