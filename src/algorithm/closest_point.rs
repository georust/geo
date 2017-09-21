use num_traits::Float;
use Point;

pub enum Closest<F: Float> {
    SinglePoint,
    Intersect(Point<F>),
    Indeterminate,
}

pub trait ClosestPoint<F: Float, Other = Point<F>> {
    fn closest_point(&self, other: &Other) -> Closest<F>;
}
