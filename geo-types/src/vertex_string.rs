use crate::{
    CoordinateType, Coordinate, Line, LineString, MultiLineString, private_utils,
};
use std::cmp::{PartialEq};
use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use num_traits::Float;

/// Container data structure's default capacity used in this mod
const DEFAULT_SIZE: usize = 4;

/// The cost function type, which is used to define the signature of the closure for calculating
/// the evaluation cost of an edge in the `VertexString`.
pub type CostFn<T> = fn(&Line<T>) -> T;

/// The type of the cost function used to calculate the edge cost
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cost<T>
where
    T: Float
{
    // Default algorithm to calculate edge cost
    Euclidean,

    // Calculate edge cost using Haversine algorithm
    Haversine,

    // Use customized cost function to calculate the edge cost
    Customize(CostFn<T>),
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

/// The struct representing the edge, aka graph relation between 2 neighboring and connected vertices.
/// This struct is mostly used internally, though we're exposing APIs to retrieve fields in case
/// applications would find them useful.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphRelation {
    // the neighbor vertex id
    vertex_id: usize,

    // the cost of the edge connecting the vertex and this neighbor
    cost: f64,
}

impl GraphRelation {
    /// Build a graph relationship between two neighboring vertices and connect them. The source
    /// vertex is where the `GraphRelation` is added to, and the target vertex's id is saved into
    /// the struct. The cost is calculated and passed in as the known value.
    pub fn new_with_cost<T: Float>(index: usize, cost: T) -> Self {
        GraphRelation {
            vertex_id: index,
            cost: cost.to_f64().unwrap_or(-1f64),
        }
    }

    /// Get the vertex id of the other end of the edge
    pub fn neighbor_id(&self) -> usize {
        self.vertex_id
    }

    /// Get the cost of traveling this edge to the other end
    pub fn edge_cost(&self) -> f64 {
        self.cost
    }
}

/// The struct representing the vertex in the `VertexString`.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vertex<T>
where
    T: Float
{
    // the node location
    coordinate: Coordinate<T>,

    // the vertex id
    id: usize,

    // vector of neighbor nodes that we can go to, stored in the form of (Destination, Distance)
    vertices: Vec<GraphRelation>,

    // the closest vertex to self
    closest_neighbor: Option<(usize, f64)>,
}

impl<T> Vertex<T>
where
    T: Float
{
    /// Create a new vertex. Parameters:
    /// - `coordinate`: the coordinate of the vertex
    /// - `id`: the id of the vertex, which is also the index of the vertex in the VertexString's
    ///         `vector` store.
    pub fn new(coordinate: Coordinate<T>, id: usize) -> Self {
        Vertex {
            coordinate,
            id,
            vertices: Vec::with_capacity(DEFAULT_SIZE),
            closest_neighbor: None,
        }
    }

    /// Get the vertex id, which also serves as the vector index in the VertexString
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Get the vertex coordinate
    pub fn get_coordinate(&self) -> Coordinate<T> {
        self.coordinate
    }

    /// Get all the neighboring vertices to the self-vertex
    pub fn get_neighbors(&self) -> &Vec<GraphRelation> {
        &self.vertices
    }

    /// Get the edge cost to the neighbor vertex
    pub fn edge_cost(&self, neighbor_id: usize) -> Option<f64> {
        for v in self.vertices.iter() {
            if v.vertex_id == neighbor_id {
                return Some(v.cost);
            }
        }

        None
    }

    fn set_vertex(&mut self, index: usize, cost: f64, is_new: bool) {
        if is_new {
            self.vertices.push(GraphRelation::new_with_cost(index, cost));
        } else {
            let size = self.vertices.len();
            for (i, v) in self.vertices.iter_mut().enumerate() {
                if v.vertex_id == index {
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

/// An undirected graph data structure that can be used to build more complex geo concepts,
/// e.g. a geo map. The struct contains a collection of connected or lone vertices, and each vertex
/// has its own struct to express its relationship with neighboring connected vertices.
///
/// The `VertexString` can be built by default from a `Line` iterator or collection container. But
/// it can also be converted from most common `Line`-based data structure, such as a vector of the `Line`s,
/// a `LineString`, a vector of `LineString`s, or a `MultiLineString`.
///
/// # Examples
///
/// Create a `VertexString` from a vector of `Line`s, where each line represent an edge
///
/// ```
/// use geo_types::{Line, VertexString};
///
/// let vec: Vec<Line<f32>> = vec![
///     Line::from([(10., 5.), (15., 10.)]),
///     Line::from([(15., 10.), (20., 15.)]),
///     Line::from([(20., 15.), (10., 5.)]),
/// ];
///
/// let graph = VertexString::from(vec);
/// //let mut it = graph.vertex_iter();
///
/// assert_eq!(it.next().unwrap().get_coordinate(), (10., 5.).into());
//  assert_eq!(it.next().unwrap().get_coordinate(), (15., 10.).into());
/// assert_eq!(it.next().unwrap().get_coordinate(), (20., 15.).into());
/// assert_eq!(it.next(), None);
/// ```
///
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VertexString<T: CoordinateType>
where
    T: Float
{
    // The store of the vertices
    vector: Vec<Vertex<T>>,

    // The index store that is used to help retrieve a vertex fast if only given the vertex coordinate
    index: HashMap<String, usize>,

    // The type of algorithm used to determine the edge cost (i.e. the distance between two vertices)
    cost_calc_type: Cost<T>,
}

impl<T> VertexString<T>
where
    T: Float
{
    /// Create a `VertexString` from the iterator of the edges connecting the vertices. If a vertex
    /// is not connected to any other vertices in the graph, then the start/end coordinate of the
    /// edge (aka the `Line<T>` object) shall be identical.
    pub fn new<C>(src: C) -> Self
    where
        C: Iterator<Item = Line<T>>
    {
        VertexString::build(src, None)
    }

    /// Create a `VertexString` from the iterator of the edges connecting the vertices.
    /// If the size of the vertex coordinates container is known, using this API will
    /// help improve the overall performance.
    pub fn new_with_size<C>(src: C, size: usize) -> Self
    where
        C: Iterator<Item = Line<T>>
    {
        VertexString::build(src, Some(size))
    }

    /// Given an edge from the VertexString, return the ids of the start and
    /// end vertices. If returned value is `None`, it means the vertex is not
    /// in the defined VertexString.
    pub fn vertex_id(&self, vertex: Coordinate<T>) -> Option<usize> {
        self.index.get(&vertex.to_string()).map(|id| id.to_owned())
    }

    /// Check if a vertex is in the VertexString graph
    pub fn contains(&self, vertex: Coordinate<T>) -> bool {
        self.index.contains_key(&vertex.to_string())
    }

    /// Given the coordinate of the vertex, return the Vertex<T> struct. If it's not contained
    /// in the VertexString graph, return `None`.
    pub fn get_vertex(&self, vertex: Coordinate<T>) -> Option<&Vertex<T>> {
        self.index
            .get(&vertex.to_string())
            .and_then(|id| {
                self.get_vertex_by_id(id.to_owned())
            })
    }

    /// Given the id of the vertex, return the Vertex<T> struct. If it's not contained
    /// in the VertexString graph, return `None`.
    pub fn get_vertex_by_id(&self, id: usize) -> Option<&Vertex<T>> {
        self.vector.get(id)
    }

    /// Set the edge cost to an arbitrary value. This shouldn't be used by anyone outside of the
    /// project-related crates.
    #[doc(hidden)]
    pub fn set_edge_cost(&mut self, edge: &Line<T>, cost: f64) {
        if edge.start == edge.end {
            // don't bother updating the sole vertex
            return;
        }

        let size = self.vector.len();

        let start_index = match self.index.get(&edge.start.to_string()) {
            Some(idx) if *idx < size => idx.to_owned(),
            _ => return,
        };

        let end_index = match self.index.get(&edge.end.to_string()) {
            Some(idx) if *idx < size => idx.to_owned(),
            _ => return,
        };

        if let Some(v) = self.vector.get_mut(start_index) {
            v.set_vertex(end_index, cost, false);
        }

        if let Some(v) = self.vector.get_mut(end_index) {
            v.set_vertex(start_index, cost, false);
        }
    }

    /// Get the edge cost. If the given edge does not exist (both vertices could be contained but
    /// not connected), return `None`.
    pub fn get_edge_cost(&self, edge: &Line<T>) -> Option<f64> {
        self.vertex_id(edge.start)
            .and_then(|start_id| {
                self.vertex_id(edge.end)
                    .map(|end_id| {
                        (start_id, end_id)
                    })
            })
            .and_then(|edge_ids| {
                self.vector
                    .get(edge_ids.0.to_owned())
                    .and_then(|v: &Vertex<T>| {
                        for neighbors in v.vertices.iter() {
                            if neighbors.vertex_id == edge_ids.1 {
                                return Some(neighbors.cost)
                            }
                        }

                        None
                    })
            })
    }

    /// Return the cost type, which is of type `Cost<T>`
    pub fn get_cost_type(&self) -> &Cost<T> {
        &self.cost_calc_type
    }

    /// Set the algorithm used to calculate the edge cost. This shouldn't be used by anyone outside
    /// of the project-related crates.
    #[doc(hidden)]
    pub fn set_cost_type(&mut self, cost: Cost<T>) {
        self.cost_calc_type = cost;
    }

    /// Collect all the edges that are present in the VertexString graph.
    pub fn edges(&self) -> Vec<Line<T>> {
        self.vector
            .iter()
            .flat_map(|v| {
                let index =
                    self.index
                        .get(&v.coordinate.to_string())
                        .expect("The vertex data is corrupted: the given coordinate does not match any of the vertices...");

                // vertex connected to other vertices, create iterator for edge in indices
                v.vertices.iter().filter_map(move |t| {
                    if t.vertex_id > *index {
                        // make sure we won't create duplicate lines
                        return Some((index.to_owned(), t.vertex_id))
                    }

                    None
                })
            })
            .map(|res|
                Line::new(self.vector[res.0].coordinate, self.vector[res.1].coordinate)
            )
            .collect()
    }

    /// Get all vertex neighbors that are connected to the queried one. Use vertex id as input.
    pub fn vertex_neighbors_by_id(&self, id: usize) -> Option<&Vec<GraphRelation>> {
        self.vector.get(id).and_then(|v| {
            Some(&v.vertices)
        })
    }

    /// Get all vertex neighbors that are connected to the queried one. Use vertex coordinate as input.
    pub fn vertex_neighbors(&self, coordinate: Coordinate<T>) -> Option<&Vec<GraphRelation>> {
        self.index.get(&coordinate.to_string()).and_then(|id| {
            self.vertex_neighbors_by_id(id.to_owned())
        })
    }

    /// Obtain an iterator that will let you iterate all the vertex from the VertexString
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
                if line.start == line.end {
                    return;
                }

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
                    a.vertex_id == b.vertex_id
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
            vector.push(Vertex::new(coordinate, pos));
            pos
        }
    }
}

// The iterator container, which can be crated from calling `vertex_iter()` on the VertexString struct
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
        assert!(!item.is_empty() && !item[0].0.is_empty());

        let estimated_size =
            item.iter().fold(0, |acc, x| acc + x.0.len() -1);

        VertexString::new_with_size(
            item.iter().flat_map(|l| {
                assert!(!l.0.is_empty());
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
    fn graph_from_line_vec() {
        let vec: Vec<Line<f32>> = vec![
            Line::from([(10., 5.), (15., 10.)]),
            Line::from([(15., 10.), (20., 15.)]),
            Line::from([(20., 15.), (10., 5.)]),
        ];

        let graph = VertexString::from(vec);
        let mut it = graph.vertex_iter();

        assert_eq!(it.next().unwrap().get_coordinate(), (10., 5.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (15., 10.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (20., 15.).into());
        assert_eq!(it.next(), None);
    }

    #[test]
    fn graph_from_linestring() {
        let line_string: LineString<f32> = vec![(0., 0.), (5., 0.), (7., 9.)].into_iter().collect();

        let graph = VertexString::from(line_string);
        let mut it = graph.vertex_iter();

        assert_eq!(it.next().unwrap().get_coordinate(), (0., 0.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (5., 0.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (7., 9.).into());
        assert_eq!(it.next(), None);

        let neighbors = graph.vertex_neighbors((5.0, 0.).into());
        assert!(neighbors.is_some());
        assert_eq!(neighbors.unwrap().len(), 2);
        assert_eq!(neighbors.unwrap()[0].vertex_id, 0);
        assert_eq!(neighbors.unwrap()[1].vertex_id, 2);
    }

    #[test]
    fn graph_from_linestring_vec() {
        let line_string_one: LineString<f32> = vec![(0., 0.), (5., 0.), (7., 9.)].into_iter().collect();
        let line_string_two: LineString<f32> = vec![(5., 0.), (8., 0.), (0., 0.)].into_iter().collect();

        let graph = VertexString::from(vec![line_string_one, line_string_two]);
        let mut it = graph.vertex_iter();

        assert_eq!(it.next().unwrap().get_coordinate(), (0., 0.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (5., 0.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (7., 9.).into());
        assert_eq!(it.next().unwrap().get_coordinate(), (8., 0.).into());
        assert_eq!(it.next(), None);

        let zero_vertex = graph.get_vertex((0., 0.).into());
        assert!(zero_vertex.is_some());
        assert_eq!(zero_vertex.unwrap().vertices.len(), 2);
        assert_eq!(zero_vertex.unwrap().vertices[0].vertex_id, 1);
        assert_eq!(zero_vertex.unwrap().vertices[1].vertex_id, 3);

        let one_vertex = graph.get_vertex_by_id(1);
        assert!(one_vertex.is_some());
        assert_eq!(one_vertex.unwrap().coordinate, (5., 0.).into());

        let three_vertex = graph.get_vertex_by_id(3);
        assert!(three_vertex.is_some());
        assert_eq!(three_vertex.unwrap().coordinate, (8., 0.).into());
    }

    #[test]
    fn neighbors() {
        let line_string: LineString<f32> = vec![(0., 0.), (5., 0.), (7., 9.)].into_iter().collect();
        let graph = VertexString::from(line_string);

        let neighbors = graph.vertex_neighbors((5.0, 0.).into());

        assert!(neighbors.is_some());
        assert_eq!(neighbors.unwrap().len(), 2);
        assert_eq!(neighbors.unwrap()[0].vertex_id, 0);
        assert_eq!(neighbors.unwrap()[1].vertex_id, 2);
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