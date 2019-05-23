use crate::{
    CoordinateType, Coordinate, Line, LineString, MultiLineString, Point, private_utils,
};
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;
use num_traits::Float;

const DEFAULT_SIZE: usize = 4;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cost<T>
where
    T: Float,
{
    Euclidean(fn(Coordinate<T>, Coordinate<T>) -> T),
    Haversine(fn(Coordinate<T>, Coordinate<T>) -> T),
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphRelation
{
    vertex_index: usize,
    cost: f64,
}

impl GraphRelation
{
    pub fn new_with_cost<T: Float>(index: usize, cost: T) -> Self {
        GraphRelation {
            vertex_index: index,
            cost: cost.to_f64().unwrap_or(-1f64),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vertex<T>
where
    T: Float + Eq + Hash
{
    // the node location
    coordinate: Coordinate<T>,

    // vector of neighbor nodes that we can go to, stored in the form of (Destination, Distance)
    vertices: Vec<GraphRelation>,

    // the closest vertex to self
    closest_vertex: Option<(usize, f64)>,
}

impl<T> Vertex<T>
where
    T: Float + Eq + Hash
{
    pub fn new(coordinate: Coordinate<T>) -> Self {
        Vertex {
            coordinate,
            vertices: Vec::with_capacity(DEFAULT_SIZE),
            closest_vertex: None,
        }
    }

    fn add_vertex(&mut self, index: usize, cost: f64) {
        self.vertices.push(GraphRelation::new_with_cost(index, cost));

        let need_update = match self.closest_vertex {
            Some((_, c)) => c > cost,
            None => true,
        };

        if need_update {
            self.closest_vertex.replace((index, cost));
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VertexString<T: CoordinateType>
where
    T: Float + Eq + Hash
{
    index: HashMap<Coordinate<T>, usize>,
    vector: Vec<Vertex<T>>,
    cost_calc_type: Cost<T>,
}

impl<T> VertexString<T>
where
    T: Float + Eq + Hash
{
    pub fn new<C>(src: C) -> Self
    where
        C: Iterator<Item = Line<T>>
    {
        VertexString::build(src, None)
    }

    pub fn new_with_size<C>(src: C, size: usize) -> Self
    where
        C: Iterator<Item = Line<T>>
    {
        VertexString::build(src, Some(size))
    }

    pub fn set_cost_calc_type(&mut self, calc_type: Cost<T>) {
        self.cost_calc_type = calc_type;

        let cost_fn = match self.cost_calc_type {
            Cost::Euclidean(function) => function,
            Cost::Haversine(function) => function,
        };

        let lines = self.lines();
        //TODO: update...
    }

    pub fn lines(&self) -> Vec<Line<T>> {
        self.vector
            .iter()
            .flat_map(|v| {
                let index = self.index.get(&v.coordinate).unwrap();
                v.vertices.iter().filter_map(move |t| {
                    if &t.vertex_index > index {
                        return Some((index.to_owned(), t.vertex_index))
                    }

                    None
                })
            })
            .map(|res|
                Line::new(self.vector[res.0].coordinate, self.vector[res.1].coordinate)
            )
            .collect()
    }

    fn build<C>(src: C, cap: Option<usize>) -> Self
    where
        C: Iterator<Item = Line<T>>
    {
        // prepare the base store
        let size_hint = if let Some(size) = cap {
            // actual capacity
            size
        } else {
            // a good guess
            DEFAULT_SIZE
        };

        let mut vector = Vec::with_capacity(size_hint);
        let mut index = HashMap::with_capacity(size_hint);

        // update the node -> edge map
        src
            .for_each(|line| {
                // find or push
                let start_index = VertexString::find_or_insert(line.start, &mut vector, &mut index);
                let end_index = VertexString::find_or_insert(line.end, &mut vector, &mut index);

                // default to calculate vertices distance using the euclidean cost
                let cost = private_utils::line_euclidean_length(line);

                if let Some(v) = vector.get_mut(start_index) {
                    v.add_vertex(end_index, cost.to_f64().unwrap_or(-1f64));
                }

                if let Some(v) = vector.get_mut(end_index) {
                    v.add_vertex(start_index, cost.to_f64().unwrap_or(-1f64));
                }
            });

        // if given a size estimation, it may be larger than what we actually need, shrink vertices
        // save spaces
        vector.shrink_to_fit();
        index.shrink_to_fit();

        // the edges may have duplicates, removing them now
        vector
            .iter_mut()
            .for_each(|val| {
                val.vertices.dedup_by(|a, b| {
                    a.vertex_index == b.vertex_index
                });
            });

        VertexString {
            vector,
            index,
            cost_calc_type: Cost::Euclidean(|start, end| {
                private_utils::line_euclidean_length(Line::new(start, end))
            }),
        }
    }

    fn find_or_insert(
        coordinate: Coordinate<T>, vector: &mut Vec<Vertex<T>>, index: &mut HashMap<Coordinate<T>, usize>
    ) -> usize {
        if let Some(pos) = index.get(&coordinate) {
            pos.to_owned()
        } else {
            let pos = vector.len();
            index.insert(coordinate, pos);
            vector.push(Vertex::new(coordinate));
            pos
        }
    }
}

impl<T> Iterator for VertexString<T>
where
    T: Float + Eq + Hash
{
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.vector.iter().map(|v| Point(v.coordinate)).next()
    }
}

impl<T> From<Vec<Line<T>>> for VertexString<T>
where
    T: Float + Eq + Hash
{
    fn from(item: Vec<Line<T>>) -> Self {
        let size = item.len();
        assert!(size > 0);

        VertexString::new_with_size(item.into_iter(), size)
    }
}

impl<T: CoordinateType> From<LineString<T>> for VertexString<T>
where
    T: Float + Eq + Hash
{
    fn from(item: LineString<T>) -> Self {
        let size = item.0.len();
        assert!(size > 0);

        VertexString::new_with_size(
            item.lines(),
            size - 1
        )
    }
}

impl<T: CoordinateType> From<Vec<LineString<T>>> for VertexString<T>
where
    T: Float + Eq + Hash
{
    fn from(item: Vec<LineString<T>>) -> Self {
        assert!(item.len() > 0 && item[0].0.len() > 0);

        let estimated_size =
            item.iter().fold(0, |acc, x| acc + x.0.len() -1);

        VertexString::new_with_size(
            item.iter().flat_map(|l| {
                assert!(l.0.len() > 0);
                l.lines()
            }),
            estimated_size
        )
    }
}

impl<T: CoordinateType> From<MultiLineString<T>> for VertexString<T>
    where
        T: Float + Eq + Hash
{
    fn from(item: MultiLineString<T>) -> Self {
        VertexString::from(item.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Line, VertexString};
}