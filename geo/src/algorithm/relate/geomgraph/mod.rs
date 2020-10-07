#![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt;

pub(crate) use edge::Edge;
pub(crate) use edge_end::{EdgeEnd, EdgeEndKey};
pub(crate) use edge_end_bundle::{EdgeEndBundle, LabeledEdgeEndBundle};
pub(crate) use edge_end_bundle_star::{EdgeEndBundleStar, LabeledEdgeEndBundleStar};
pub(crate) use edge_intersection::EdgeIntersection;
pub(crate) use geometry_graph::GeometryGraph;
pub(crate) use intersection_matrix::IntersectionMatrix;
pub(crate) use label::Label;
pub(crate) use line_intersector::{LineIntersection, LineIntersector};
pub(crate) use node::CoordNode;
use planar_graph::PlanarGraph;
pub(crate) use quadrant::Quadrant;
pub(crate) use robust_line_intersector::RobustLineIntersector;
use topology_position::TopologyPosition;

use crate::dimensions::Dimensions;
pub use crate::utils::CoordPos;

mod edge;
mod edge_end;
mod edge_end_bundle;
mod edge_end_bundle_star;
mod edge_intersection;
mod geometry_graph;
pub(crate) mod index;
mod label;
mod node;
pub(crate) mod node_map;
mod planar_graph;
mod quadrant;
mod topology_position;

pub(crate) mod intersection_matrix;
mod line_intersector;
mod robust_line_intersector;

/// Position relative to a point
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    On,
    Left,
    Right,
}
