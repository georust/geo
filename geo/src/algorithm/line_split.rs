use geo_types::CoordNum;

use crate::coords_iter::CoordsIter;
use crate::{CoordFloat, EuclideanLength, Line, LineString, Point};
use std::ops::AddAssign;

///
///
///
pub trait LineSplit<Scalar: CoordNum> {
    type Output;

    fn line_split(&self, fraction: Scalar) -> (Option<Self::Output>, Option<Self::Output>);
    fn line_split_twice(
        &self,
        start_fraction: Scalar,
        end_fraction: Scalar,
    ) -> (
        Option<Self::Output>,
        Option<Self::Output>,
        Option<Self::Output>,
    );
}

impl<Scalar> LineSplit<Scalar> for Line<Scalar>
where
    Scalar: CoordFloat,
{
    type Output = Line<Scalar>;

    fn line_split(&self, fraction: Scalar) -> (Option<Self::Output>, Option<Self::Output>) {
        let fraction = fraction.max(Scalar::zero()).min(Scalar::one());
        if fraction == Scalar::zero() {
            (None, Some(self.clone()))
        } else if fraction == Scalar::one() {
            (Some(self.clone()), None)
        } else {
            let new_midpoint = self.start + self.delta() * fraction;
            (
                Some(Line::new(self.start, new_midpoint)),
                Some(Line::new(new_midpoint, self.end)),
            )
        }
    }

    fn line_split_twice(
        &self,
        start_fraction: Scalar,
        end_fraction: Scalar,
    ) -> (
        Option<Self::Output>,
        Option<Self::Output>,
        Option<Self::Output>,
    ) {
        // forgive the user for passing in the wrong order
        // because it simplifies the interface of the output type
        let (start_fraction, end_fraction) = if start_fraction > end_fraction {
            (end_fraction, start_fraction)
        } else {
            (start_fraction, end_fraction)
        };
        // TODO: check for nan
        let second_fraction = (end_fraction - start_fraction) / (Scalar::one() - start_fraction);
        match self.line_split(start_fraction) {
            (Some(first_line), Some(second_line)) => {
                match second_line.line_split(second_fraction) {
                    (Some(second_line), Some(third_line)) => {
                        (Some(first_line), Some(second_line), Some(third_line))
                    }
                    (Some(second_line), None) => (Some(first_line), Some(second_line), None),
                    (None, Some(third_line)) => (Some(first_line), None, Some(third_line)),
                    (None, None) => (Some(first_line), None, None),
                }
            }
            (None, Some(second_line)) => match second_line.line_split(second_fraction) {
                (Some(second_line), Some(third_line)) => {
                    (None, Some(second_line), Some(third_line))
                }
                (Some(second_line), None) => (None, Some(second_line), None), // Never
                (None, Some(third_line)) => (None, None, Some(third_line)),
                (None, None) => (None, None, None),
            },
            (Some(first_line), None) => (Some(first_line), None, None),
            (None, None) => (None, None, None),
        }
    }
}

impl<Scalar> LineSplit<Scalar> for LineString<Scalar>
where
    Scalar: CoordFloat,
{
    type Output = LineString<Scalar>;

    fn line_split(&self, fraction: Scalar) -> (Option<Self::Output>, Option<Self::Output>) {
        todo!()
    }

    fn line_split_twice(
        &self,
        start_fraction: Scalar,
        end_fraction: Scalar,
    ) -> (
        Option<Self::Output>,
        Option<Self::Output>,
        Option<Self::Output>,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{coord, point};
    use crate::{ClosestPoint, LineLocatePoint};
    use num_traits::Float;

    #[test]
    fn test_linestring_slice() {
        todo!();
    }
}
