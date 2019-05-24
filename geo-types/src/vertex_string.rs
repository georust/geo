use crate::{
    CoordinateType, Coordinate, Line, LineString, MultiLineString, private_utils,
};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use num_traits::Float;

// Mean radius of Earth in meters
const DEFAULT_SIZE: usize = 4;

pub type CostFn<T> = fn(&Line<T>) -> T;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cost<T>
where
    T: Float
{
    Euclidean,             // Use default euclidean distance for calculation
    Haversine,             // Obtain provided Haversine cost function
    Customize(CostFn<T>),  // Define customized cost function
}

impl<T> fmt::Debug for Cost<T>
where
    T: Float,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Cost::Euclidean => "Euclidean",
            Cost::Haversine => "Haversine",
            Cost::Customize(_) => "Customized_Cost_Function",
        };

        write!(f, "{}", name)
    }
}

impl<T> PartialEq for Cost<T>
where
    T: Float,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Cost::Euclidean, Cost::Euclidean) => true,
            (Cost::Haversine, Cost::Haversine) => true,
            (Cost::Customize(func_self), Cost::Customize(func_other)) => {
                std::ptr::eq(func_self, func_other)
            },
            _ => false
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphRelation {
    vertex_index: usize,
    cost: f64,
}

impl GraphRelation {
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
    T: Float
{
    // the node location
    coordinate: Coordinate<T>,

    // vector of neighbor nodes that we can go to, stored in the form of (Destination, Distance)
    vertices: Vec<GraphRelation>,

    // the closest vertex to self
    closest_neighbor: Option<(usize, f64)>,
}

impl<T> Vertex<T>
where
    T: Float
{
    pub fn new(coordinate: Coordinate<T>) -> Self {
        Vertex {
            coordinate,
            vertices: Vec::with_capacity(DEFAULT_SIZE),
            closest_neighbor: None,
        }
    }

    fn set_vertex(&mut self, index: usize, cost: f64, is_new: bool) {
        if is_new {
            self.vertices.push(GraphRelation::new_with_cost(index, cost));
        } else {
            let size = self.vertices.len();
            for (i, v) in self.vertices.iter_mut().enumerate() {
                if v.vertex_index == index {
                    v.cost = cost;
                    break;
                }

                // if last of the vertices won't match, we're trying to update a vertex not
                // connected to this vertex, panic.
                assert!(i < size - 1, "trying to update a neighbor vertex not connected to the current one...");
            }
        }

        let need_update = match self.closest_neighbor {
            Some((_, c)) => c > cost,
            None => true,
        };

        if need_update {
            self.closest_neighbor.replace((index, cost));
        }
    }
}

impl<T> PartialEq for  Vertex<T>
where
    T: Float
{
    fn eq(&self, other: &Self) -> bool {
        self.coordinate == other.coordinate
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VertexString<T: CoordinateType>
where
    T: Float
{
    vector: Vec<Vertex<T>>,
    index: HashMap<String, usize>,
    cost_calc_type: Cost<T>,
}

impl<T> VertexString<T>
where
    T: Float
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

    pub fn set_edge_cost(&mut self, edge: &Line<T>, cost: f64) {
        let (start_key, end_key) = (edge.start.to_string(), edge.end.to_string());
        let size = self.vector.len();

        let start_index = match self.index.get(&start_key) {
            Some(idx) if idx < &size => idx.to_owned(),
            _ => return,
        };

        let end_index = match self.index.get(&end_key) {
            Some(idx) if idx < &size => idx.to_owned(),
            _ => return,
        };

        if let Some(v) = self.vector.get_mut(start_index) {
            v.set_vertex(end_index, cost, false);
        }

        if let Some(v) = self.vector.get_mut(end_index) {
            v.set_vertex(start_index, cost, false);
        }
    }

    pub fn get_cost_type(&self) -> &Cost<T> {
        &self.cost_calc_type
    }

    pub fn set_cost_type(&mut self, cost: Cost<T>) {
        self.cost_calc_type = cost;
    }

    pub fn edges(&self) -> Vec<Line<T>> {
        self.vector
            .iter()
            .flat_map(|v| {
                let index = self.index.get(&v.coordinate.to_string()).unwrap();
                v.vertices.iter().filter_map(move |t| {
                    if &t.vertex_index > index {
                        // make sure we won't create duplicate lines
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

    pub fn vertex_iter(&self) -> VertexIter<T> {
        VertexIter(self.vector.iter())
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
                let cost = private_utils::line_euclidean_length(line).to_f64().unwrap_or(-1f64);

                if let Some(v) = vector.get_mut(start_index) {
                    v.set_vertex(end_index, cost, true);
                }

                if let Some(v) = vector.get_mut(end_index) {
                    v.set_vertex(start_index, cost, true);
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
            cost_calc_type: Cost::Euclidean,
        }
    }

    fn find_or_insert(
        coordinate: Coordinate<T>,
        vector: &mut Vec<Vertex<T>>,
        index: &mut HashMap<String, usize>
    ) -> usize
    {
        let key = coordinate.to_string();

        if let Some(pos) = index.get(&key) {
            pos.to_owned()
        } else {
            let pos = vector.len();
            index.insert(key, pos);
            vector.push(Vertex::new(coordinate));
            pos
        }
    }
}

pub struct VertexIter<'a, T: Float + 'a>(::std::slice::Iter<'a, Vertex<T>>);

impl<'a, T: Float> Iterator for VertexIter<'a, T> {
    type Item = &'a Vertex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> From<Vec<Line<T>>> for VertexString<T>
where
    T: Float
{
    fn from(item: Vec<Line<T>>) -> Self {
        let size = item.len();
        assert!(size > 0);

        VertexString::new_with_size(item.into_iter(), size)
    }
}

impl<T: CoordinateType> From<LineString<T>> for VertexString<T>
where
    T: Float
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
    T: Float
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
        T: Float
{
    fn from(item: MultiLineString<T>) -> Self {
        VertexString::from(item.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Line, VertexString};

    #[test]
    fn graph() {
        let graph = VertexString::from(
            vec![
                Line::new(
                    Coordinate { x: 10f64, y: 5f64 },
                    Coordinate { x: 15f64, y: 10f64 }
                ),
                Line::new(
                    Coordinate { x: 15f64, y: 10f64 },
                    Coordinate { x: 20f64, y: 15f64 }
                ),
                Line::new(
                    Coordinate { x: 20f64, y: 15f64 },
                    Coordinate { x: 10f64, y: 5f64 }
                ),
            ]
        );

        let mut it = graph.vertex_iter();

        assert_eq!(it.next().unwrap().coordinate, (10., 5.).into());
        assert_eq!(it.next().unwrap().coordinate, (15., 10.).into());
        assert_eq!(it.next().unwrap().coordinate, (20., 15.).into());
        assert_eq!(it.next(), None);
    }

    #[test]
    fn edges() {
        let graph = VertexString::from(
            vec![
                Line::new(
                    Coordinate { x: 10f64, y: 5f64 },
                    Coordinate { x: 15f64, y: 10f64 }
                ),
                Line::new(
                    Coordinate { x: 15f64, y: 10f64 },
                    Coordinate { x: 20f64, y: 15f64 }
                ),
                Line::new(
                    Coordinate { x: 20f64, y: 15f64 },
                    Coordinate { x: 10f64, y: 5f64 }
                ),
            ]
        );

        let edges = graph.edges();
        assert_eq!(edges.len(), 3);

        assert_eq!(edges[0], Line::new(
            Coordinate { x: 10f64, y: 5f64 },
            Coordinate { x: 15f64, y: 10f64 }
        ));
        assert_eq!(edges[1], Line::new(
            Coordinate { x: 10f64, y: 5f64 },
            Coordinate { x: 20f64, y: 15f64 }
        ));
        assert_eq!(edges[2], Line::new(
            Coordinate { x: 15f64, y: 10f64 },
            Coordinate { x: 20f64, y: 15f64 }
        ));
    }
}