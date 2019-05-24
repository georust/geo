use crate::{
    Line, VertexString, Cost, CostFn,
    euclidean_length::EuclideanLength, haversine_distance::HaversineDistance,
};
use num_traits::{Float, FromPrimitive};

pub trait CostCalc<T>
where
    T: Float
{
    fn calc_edge_cost(&mut self, cost_type: Cost<T>);
}

impl<T> CostCalc<T> for VertexString<T>
where
    T: Float + FromPrimitive,
{
    fn calc_edge_cost(&mut self, cost_type: Cost<T>) {
        if self.get_cost_type() == &cost_type {
            // same cost type, no need to calc again
            return;
        }

        let cost_fn: CostFn<T> = match cost_type {
            Cost::Euclidean => |line: &Line<T>| {
                line.euclidean_length()
            },
            Cost::Haversine => |line: &Line<T>| {
                let (start, end) = line.points();
                start.haversine_distance(&end)
            },
            Cost::Customize(func) => func,
        };

        self.edges()
            .iter()
            .for_each(|line| {
                self.set_edge_cost(line, cost_fn(line).to_f64().unwrap_or(-1.));
            });

        self.set_cost_type(cost_type);
    }
}