use super::winding_order::*;
use crate::{CoordinateType, LineString};
use num_traits::Float;

pub trait RobustWinding<T>: Winding<T>
where
    T: CoordinateType,
{
    /// Return the winding order of this object
    fn robust_winding_order(&self) -> Option<WindingOrder>;
}

/// Compute index of the lexicographically least point.
/// Should only be called on a non-empty slice.
fn lexicographically_least_index<T: Copy + PartialOrd>(pts: &[T]) -> usize {
    assert!(pts.len() > 0);

    let mut min: Option<(usize, T)> = None;
    for (i, pt) in pts.iter().enumerate() {
        if let Some((_, min_pt)) = min {
            if pt < &min_pt {
                min = Some( (i, *pt) )
            }
        } else {
            min = Some( (i, *pt) )
        }
    }

    min.unwrap().0
}

impl<T: CoordinateType + Float> RobustWinding<T> for LineString<T> {
    fn robust_winding_order(&self) -> Option<WindingOrder> {
        // If linestring has at most 2 points, it is either
        // not closed, or is the same point. Either way, the
        // WindingOrder is unspecified.
        if self.num_coords() < 3 { return None; }

        // Open linestrings do not have a winding order.
        if !self.is_closed() { return None; }

        let increment = |x: &mut usize| {
            *x += 1;
            if *x >= self.num_coords() { *x = 0; }
        };

        let decrement = |x: &mut usize| {
            if *x == 0 {
                *x = self.num_coords() - 1;
            } else { *x -= 1; }
        };

        let i = lexicographically_least_index(&self.0);

        let mut next = i;
        increment(&mut next);
        while self.0[next] == self.0[i] {
            if next == i {
                // We've looped too much. There aren't
                // enough unique coords to compute orientation.
                return None;
            }
            increment(&mut next);
        }

        let mut prev = i;
        decrement(&mut prev);
        while self.0[prev] == self.0[i] {
            if prev == i {
                // We've looped too much. There aren't enough
                // unique coords to compute orientation.
                //
                // Note: we actually don't need this check as
                // the previous loop succeeded, and so we have
                // at least two distinct elements in the list
                return None;
            }
            decrement(&mut prev);
        }

        use robust::{Coord, orient2d};
        use num_traits::NumCast;
        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from( self.0[prev].x ).unwrap(),
                y: <f64 as NumCast>::from( self.0[prev].y ).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from( self.0[i].x ).unwrap(),
                y: <f64 as NumCast>::from( self.0[i].y ).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from( self.0[next].x ).unwrap(),
                y: <f64 as NumCast>::from( self.0[next].y ).unwrap(),
            },
        );

        if orientation < 0. {
            Some(WindingOrder::Clockwise)
        } else if orientation > 0. {
            Some(WindingOrder::CounterClockwise)
        } else {
            None
        }

    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;

    #[test]
    fn robust_winding_order() {
        // 3 points forming a triangle
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // That triangle, but in clockwise ordering
        let cw_line = LineString::from(vec![a.0, c.0, b.0, a.0]);
        // That triangle, but in counterclockwise ordering
        let ccw_line = LineString::from(vec![a.0, b.0, c.0, a.0]);

        // Verify open linestrings return None
        assert!(LineString::from(vec![a.0, b.0, c.0])
            .robust_winding_order()
            .is_none());

        assert_eq!(cw_line.robust_winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(cw_line.is_cw(), true);
        assert_eq!(cw_line.is_ccw(), false);
        assert_eq!(
            ccw_line.robust_winding_order(),
            Some(WindingOrder::CounterClockwise)
        );
        assert_eq!(ccw_line.is_cw(), false);
        assert_eq!(ccw_line.is_ccw(), true);

        let cw_points1: Vec<_> = cw_line.points_cw().collect();
        assert_eq!(cw_points1.len(), 4);
        assert_eq!(cw_points1[0], a);
        assert_eq!(cw_points1[1], c);
        assert_eq!(cw_points1[2], b);
        assert_eq!(cw_points1[3], a);

        let ccw_points1: Vec<_> = cw_line.points_ccw().collect();
        assert_eq!(ccw_points1.len(), 4);
        assert_eq!(ccw_points1[0], a);
        assert_eq!(ccw_points1[1], b);
        assert_eq!(ccw_points1[2], c);
        assert_eq!(ccw_points1[3], a);

        assert_ne!(cw_points1, ccw_points1);

        let cw_points2: Vec<_> = ccw_line.points_cw().collect();
        let ccw_points2: Vec<_> = ccw_line.points_ccw().collect();

        // cw_line and ccw_line are wound differently, but the ordered winding iterator should have
        // make them similar
        assert_eq!(cw_points2, cw_points2);
        assert_eq!(ccw_points2, ccw_points2);

        // test make_clockwise_winding
        let mut new_line1 = ccw_line.clone();
        new_line1.make_cw_winding();
        assert_eq!(new_line1.robust_winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(new_line1, cw_line);
        assert_ne!(new_line1, ccw_line);

        // test make_counterclockwise_winding
        let mut new_line2 = cw_line.clone();
        new_line2.make_ccw_winding();
        assert_eq!(
            new_line2.robust_winding_order(),
            Some(WindingOrder::CounterClockwise)
        );
        assert_ne!(new_line2, cw_line);
        assert_eq!(new_line2, ccw_line);
    }
}
