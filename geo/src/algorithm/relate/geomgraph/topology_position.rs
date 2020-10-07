use super::{CoordPos, Direction};

use std::fmt;

/// A `TopologyPosition` is the labelling of a graph component's topological relationship to a
/// single Geometry for each of the component's [`Direction`s](Direction).
///
/// If the graph component is an _area_ edge, there is a position for each [`Direction`]:
/// - [`On`](Direction::On): on the edge
/// - [`Left`](Direction::Left): left-hand side of the edge
/// - [`Right`](Direction::Right): right-hand side
///
/// If the parent component is a _line_ edge or a node (a point), there is a single
/// topological relationship attribute for the [`On`](Direction::On) position.
///
/// See [`CoordPos`] for the possible values.
#[derive(Copy, Clone)]
pub(crate) enum TopologyPosition {
    Area {
        on: Option<CoordPos>,
        left: Option<CoordPos>,
        right: Option<CoordPos>,
    },
    LineOrPoint {
        on: Option<CoordPos>,
    },
}

impl fmt::Debug for TopologyPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_position(position: &Option<CoordPos>, f: &mut fmt::Formatter) -> fmt::Result {
            match position {
                Some(CoordPos::Inside) => write!(f, "i"),
                Some(CoordPos::OnBoundary) => write!(f, "b"),
                Some(CoordPos::Outside) => write!(f, "e"),
                None => write!(f, "_"),
            }
        }
        match self {
            Self::LineOrPoint { on } => fmt_position(on, f)?,
            Self::Area { on, left, right } => {
                fmt_position(left, f)?;
                fmt_position(on, f)?;
                fmt_position(right, f)?;
            }
        }
        Ok(())
    }
}

impl TopologyPosition {
    pub fn area(on: CoordPos, left: CoordPos, right: CoordPos) -> Self {
        Self::Area {
            on: Some(on),
            left: Some(left),
            right: Some(right),
        }
    }

    pub fn empty_area() -> Self {
        Self::Area {
            on: None,
            left: None,
            right: None,
        }
    }

    pub fn line_or_point(on: CoordPos) -> Self {
        Self::LineOrPoint { on: Some(on) }
    }

    pub fn empty_line_or_point() -> Self {
        Self::LineOrPoint { on: None }
    }

    pub fn get(&self, direction: Direction) -> Option<CoordPos> {
        match (direction, self) {
            (Direction::Left, Self::Area { left, .. }) => *left,
            (Direction::Right, Self::Area { right, .. }) => *right,
            (Direction::On, Self::LineOrPoint { on }) | (Direction::On, Self::Area { on, .. }) => {
                *on
            }
            (_, Self::LineOrPoint { .. }) => {
                panic!("LineOrPoint only has a position for `Direction::On`")
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            Self::LineOrPoint { on: None }
                | Self::Area {
                    on: None,
                    left: None,
                    right: None,
                }
        )
    }

    pub fn is_any_empty(&self) -> bool {
        !matches!(
            self,
            Self::LineOrPoint { on: Some(_) }
                | Self::Area {
                    on: Some(_),
                    left: Some(_),
                    right: Some(_),
                }
        )
    }

    pub fn is_area(&self) -> bool {
        matches!(self, Self::Area { .. })
    }

    pub fn is_line(&self) -> bool {
        matches!(self, Self::LineOrPoint { .. })
    }

    pub fn flip(&mut self) {
        match self {
            Self::LineOrPoint { .. } => {}
            Self::Area { left, right, .. } => {
                std::mem::swap(left, right);
            }
        }
    }

    pub fn set_all_positions(&mut self, position: CoordPos) {
        match self {
            Self::LineOrPoint { on } => {
                *on = Some(position);
            }
            Self::Area { on, left, right } => {
                *on = Some(position);
                *left = Some(position);
                *right = Some(position);
            }
        }
    }

    pub fn set_all_positions_if_empty(&mut self, position: CoordPos) {
        match self {
            Self::LineOrPoint { on } => {
                if on.is_none() {
                    *on = Some(position);
                }
            }
            Self::Area { on, left, right } => {
                if on.is_none() {
                    *on = Some(position);
                }
                if left.is_none() {
                    *left = Some(position);
                }
                if right.is_none() {
                    *right = Some(position);
                }
            }
        }
    }

    pub fn set_position(&mut self, direction: Direction, position: CoordPos) {
        match (direction, self) {
            (Direction::On, Self::LineOrPoint { on }) => *on = Some(position),
            (_, Self::LineOrPoint { .. }) => {
                panic!("invalid assignment dimensions for Self::Line")
            }
            (Direction::On, Self::Area { on, .. }) => *on = Some(position),
            (Direction::Left, Self::Area { left, .. }) => *left = Some(position),
            (Direction::Right, Self::Area { right, .. }) => *right = Some(position),
        }
    }

    pub fn set_on_position(&mut self, position: CoordPos) {
        match self {
            Self::LineOrPoint { on } | Self::Area { on, .. } => {
                *on = Some(position);
            }
        }
    }

    pub fn set_locations(&mut self, new_on: CoordPos, new_left: CoordPos, new_right: CoordPos) {
        match self {
            Self::LineOrPoint { .. } => {
                error!("invalid assignment dimensions for {:?}", self);
                debug_assert!(false, "invalid assignment dimensions for {:?}", self);
            }
            Self::Area { on, left, right } => {
                *on = Some(new_on);
                *left = Some(new_left);
                *right = Some(new_right);
            }
        }
    }
}
