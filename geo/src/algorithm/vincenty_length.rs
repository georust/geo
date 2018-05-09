use num_traits::{Float, FromPrimitive};

use ::{Line, LineString, MultiLineString};
use algorithm::vincenty_distance::VincentyDistance;

pub trait VincentyLength<T, RHS = Self> {
    fn vincenty_length(&self) -> T;
}

impl<T> VincentyLength<T> for Line<T>
    where T: Float + FromPrimitive
{
    /// The units of the returned value is meters.
    fn vincenty_length(&self) -> T {
        self.start.vincenty_distance(&self.end)
    }
}

impl<T> VincentyLength<T> for LineString<T>
    where T: Float + FromPrimitive
{
    fn vincenty_length(&self) -> T {
        self.0.windows(2)
              .fold(T::zero(), |total_length, p| total_length + p[0].vincenty_distance(&p[1]))
    }
}

impl<T> VincentyLength<T> for MultiLineString<T>
    where T: Float + FromPrimitive
{
    fn vincenty_length(&self) -> T {
        self.0.iter().fold(T::zero(), |total, line| total + line.vincenty_length())
    }
}
