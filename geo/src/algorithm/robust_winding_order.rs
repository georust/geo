use super::winding_order::*;
use super::kernels::*;
use crate::{CoordinateType, LineString};

pub trait RobustWinding {
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

impl<T, K> RobustWinding for LineString<T>
where T: CoordinateType + HasKernel<Ker = K>,
      K: Kernel<Scalar = T>
{
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

        K::orient2d(
            self.0[prev],
            self.0[i],
            self.0[next]
        )

    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;
    use crate::algorithm::kernels::*;

    #[test]
    fn robust_winding_float() {
        // 3 points forming a triangle
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // That triangle, but in clockwise ordering
        let cw_line = LineString::from(vec![a.0, c.0, b.0, a.0]);
        // That triangle, but in counterclockwise ordering
        let ccw_line = LineString::from(vec![a.0, b.0, c.0, a.0]);

        // Verify open linestrings return None
        let open_line = LineString::from(vec![a.0, b.0, c.0]);
        assert!(
            open_line.robust_winding_order()
                .is_none()
        );

        assert_eq!(
            cw_line.robust_winding_order(),
            Some(WindingOrder::Clockwise)
        );
        assert_eq!(
            ccw_line.robust_winding_order(),
            Some(WindingOrder::CounterClockwise)
        );

    }

    #[test]
    fn robust_winding_integers() {
        // 3 points forming a triangle
        let a = Point::new(0i64, 0);
        let b = Point::new(2, 0);
        let c = Point::new(1, 2);

        // That triangle, but in clockwise ordering
        let cw_line = LineString::from(vec![a.0, c.0, b.0, a.0]);
        // That triangle, but in counterclockwise ordering
        let ccw_line = LineString::from(vec![a.0, b.0, c.0, a.0]);

        // Verify open linestrings return None
        let open_line = LineString::from(vec![a.0, b.0, c.0]);
        assert!(
            open_line.robust_winding_order()
                .is_none()
        );

        assert_eq!(
            cw_line.robust_winding_order(),
            Some(WindingOrder::Clockwise)
        );
        assert_eq!(
            ccw_line.robust_winding_order(),
            Some(WindingOrder::CounterClockwise)
        );

    }
}
