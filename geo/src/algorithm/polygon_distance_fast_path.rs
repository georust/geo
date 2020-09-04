use crate::algorithm::extremes::ExtremeIndices;
use crate::kernels::*;
use crate::prelude::*;
use crate::{Line, Point, Polygon, Triangle};
use num_traits::float::FloatConst;
use num_traits::{Float, Signed};

// These are helper functions for the "fast path" of Polygon-Polygon distance
// They use the rotating calipers method to speed up calculations.
// Tests for these functions are in the Distance module

/// Calculate the minimum distance between two disjoint and linearly separable convex polygons
/// using the rotating calipers method
pub(crate) fn min_poly_dist<T>(poly1: &Polygon<T>, poly2: &Polygon<T>) -> T
where
    T: Float + FloatConst + Signed + HasKernel,
{
    let poly1_extremes = poly1.extreme_indices().unwrap();
    let poly2_extremes = poly2.extreme_indices().unwrap();
    let ymin1 = Point(poly1.exterior().0[poly1_extremes.ymin]);
    let ymax2 = Point(poly2.exterior().0[poly2_extremes.ymax]);

    let mut state = Polydist {
        poly1,
        poly2,
        dist: T::infinity(),
        ymin1,
        ymax2,
        // initial polygon 1 min y idx
        p1_idx: poly1_extremes.ymin,
        // initial polygon 2 max y idx
        q2_idx: poly2_extremes.ymax,
        // set p1 and q2 to p1ymin and p2ymax initially
        p1: ymin1,
        q2: ymax2,
        p1next: Point::new(T::zero(), T::zero()),
        q2next: Point::new(T::zero(), T::zero()),
        p1prev: Point::new(T::zero(), T::zero()),
        q2prev: Point::new(T::zero(), T::zero()),
        alignment: None,
        ap1: T::zero(),
        aq2: T::zero(),
        start: None,
        finished: false,
        ip1: false,
        iq2: false,
        slope: T::zero(),
        vertical: false,
    };
    while !state.finished {
        nextpoints(&mut state);
        computemin(&mut state);
    }
    state.dist
}

/// Minimum distance between a vertex and an imaginary line drawn from p to q
fn vertex_line_distance<T>(v: Point<T>, p: Point<T>, q: Point<T>) -> T
where
    T: Float,
{
    v.euclidean_distance(&Line::new(p.0, q.0))
}

/// Wrap-around previous Polygon index
fn prev_vertex<T>(poly: &Polygon<T>, current_vertex: usize) -> usize
where
    T: Float,
{
    (current_vertex + (poly.exterior().0.len() - 1) - 1) % (poly.exterior().0.len() - 1)
}

/// Wrap-around next Polygon index
fn next_vertex<T>(poly: &Polygon<T>, current_vertex: usize) -> usize
where
    T: Float,
{
    (current_vertex + 1) % (poly.exterior().0.len() - 1)
}

#[derive(Debug)]
enum AlignedEdge {
    VertexP,
    VertexQ,
    Edge,
}

/// Distance-finding state
#[derive(Debug)]
pub(crate) struct Polydist<'a, T>
where
    T: Float,
    T: 'a,
{
    poly1: &'a Polygon<T>,
    poly2: &'a Polygon<T>,
    dist: T,
    ymin1: Point<T>,
    p1_idx: usize,
    ymax2: Point<T>,
    q2_idx: usize,
    p1: Point<T>,
    q2: Point<T>,
    p1next: Point<T>,
    q2next: Point<T>,
    p1prev: Point<T>,
    q2prev: Point<T>,
    alignment: Option<AlignedEdge>,
    ap1: T,
    aq2: T,
    start: Option<bool>,
    finished: bool,
    ip1: bool,
    iq2: bool,
    slope: T,
    vertical: bool,
}

// much of the following code is ported from Java, copyright 1999 Hormoz Pirzadeh, available at:
// http://web.archive.org/web/20150330010154/http://cgm.cs.mcgill.ca/%7Eorm/rotcal.html
fn unitvector<T>(slope: &T, poly: &Polygon<T>, p: Point<T>, idx: usize) -> Point<T>
where
    T: Float + Signed,
{
    let tansq = slope.powi(2);
    let cossq = T::one() / (T::one() + tansq);
    let sinsq = T::one() - cossq;
    let mut cos = T::zero();
    let mut sin;
    let pnext = poly.exterior().0[next_vertex(poly, idx)];
    let pprev = poly.exterior().0[prev_vertex(poly, idx)];
    let clockwise = Point(pprev).cross_prod(Point(p.0), Point(pnext)) < T::zero();
    let slope_prev;
    let slope_next;
    // Slope isn't 0, things are complicated
    if *slope != T::zero() {
        cos = cossq.sqrt();
        sin = sinsq.sqrt();
        if pnext.x > p.x() {
            if pprev.x > p.x() {
                if pprev.y >= p.y() && pnext.y >= p.y() {
                    if *slope > T::zero() {
                        slope_prev = Line::new(p.0, pprev).slope();
                        if clockwise && *slope <= slope_prev || !clockwise && *slope >= slope_prev {
                            cos = -cos;
                            sin = -sin;
                        } else if clockwise {
                            cos = -cos;
                        } else {
                            sin = -sin;
                        }
                    }
                } else if pprev.y <= p.y() && pnext.y <= p.y() {
                    if *slope > T::zero() {
                        if !clockwise {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else {
                        slope_prev = Line::new(p.0, pprev).slope();
                        slope_next = Line::new(p.0, pnext).slope();
                        if clockwise {
                            if *slope <= slope_prev {
                                cos = -cos;
                            } else {
                                sin = -sin;
                            }
                        } else if *slope <= slope_next {
                            sin = -sin;
                        } else {
                            cos = -cos;
                        }
                    }
                } else if *slope > T::zero() {
                    if !clockwise {
                        cos = -cos;
                        sin = -sin;
                    }
                } else if clockwise {
                    cos = -cos;
                } else {
                    sin = -sin;
                }
            } else if *slope < T::zero() {
                sin = -sin;
            }
        } else if pnext.x < p.x() {
            if pprev.x < p.x() {
                if pprev.y >= p.y() && pnext.y >= p.y() {
                    if *slope > T::zero() {
                        if clockwise {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else {
                        slope_prev = Line::new(p.0, pprev).slope();
                        slope_next = Line::new(p.0, pnext).slope();
                        if clockwise {
                            if *slope <= slope_prev {
                                sin = -sin;
                            } else {
                                cos = -cos;
                            }
                        } else if *slope <= slope_next {
                            cos = -cos;
                        } else {
                            sin = -sin;
                        }
                    }
                } else if pprev.y <= p.y() && pnext.y <= p.y() {
                    if *slope > T::zero() {
                        slope_next = Line::new(p.0, pnext).slope();
                        if *slope >= slope_next {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else if clockwise {
                        sin = -sin;
                    } else {
                        cos = -cos;
                    }
                } else if *slope > T::zero() {
                    if clockwise {
                        cos = -cos;
                        sin = -sin;
                    }
                } else if clockwise {
                    sin = -sin;
                } else {
                    cos = -cos;
                }
            } else {
                //pprev.x() >= p.x()
                cos = -cos;
                if *slope > T::zero() {
                    sin = -sin;
                }
            }
        } else if pprev.x > p.x() {
            cos = -cos;
            if *slope > T::zero() {
                sin = -sin;
            }
        } else if *slope < T::zero() {
            sin = -sin;
        }
    } else {
        // Slope is 0, things are fairly simple
        sin = T::zero();
        if pnext.x > p.x() {
            cos = T::one();
        } else if pnext.x < p.x() {
            cos = -T::one();
        } else if pnext.x == p.x() {
            if pprev.x < p.x() {
                cos = T::one();
            } else {
                cos = -T::one();
            }
        }
    }
    Point::new(
        p.x() + T::from(100).unwrap() * cos,
        p.y() + T::from(100).unwrap() * sin,
    )
}

/// Perpendicular unit vector of a vertex and a unit vector
fn unitpvector<T>(p: Point<T>, u: Point<T>) -> Point<T>
where
    T: Float,
{
    let hundred = T::from(100).unwrap();
    let vertical = p.x() == u.x();
    let slope = if vertical || p.y() == u.y() {
        T::zero()
    } else {
        Line::new(p, u).slope()
    };
    let upx;
    let upy;
    if vertical {
        upy = p.y();
        if u.y() > p.y() {
            upx = p.x() + hundred;
        } else {
            upx = p.x() - hundred;
        }
        Point::new(upx, upy)
    } else if slope == T::zero() {
        upx = p.x();
        if u.x() > p.x() {
            upy = p.y() - hundred;
        } else {
            upy = p.y() + hundred;
        }
        Point::new(upx, upy)
    } else {
        // Not a special case
        let sperp = -T::one() / slope;
        let tansq = sperp * sperp;
        let cossq = T::one() / (T::one() + tansq);
        let sinsq = T::one() - cossq;
        let mut cos = cossq.sqrt();
        let mut sin = sinsq.sqrt();
        if u.x() > p.x() {
            sin = -sin;
            if slope < T::zero() {
                cos = -cos;
            }
        } else if slope > T::zero() {
            cos = -cos;
        }
        Point::new(p.x() + hundred * cos, p.y() + hundred * sin)
    }
}

/// Angle between a vertex and an edge
fn vertex_line_angle<T>(poly: &Polygon<T>, p: Point<T>, m: &T, vertical: bool, idx: usize) -> T
where
    T: Float + FloatConst + Signed,
{
    let hundred = T::from(100).unwrap();
    let pnext = poly.exterior().0[next_vertex(poly, idx)];
    let pprev = poly.exterior().0[prev_vertex(poly, idx)];
    let clockwise = Point(pprev).cross_prod(Point(p.0), Point(pnext)) < T::zero();
    let punit;
    if !vertical {
        punit = unitvector(m, poly, p, idx);
    } else if clockwise {
        if p.x() > pprev.x {
            punit = Point::new(p.x(), p.y() - hundred);
        } else if p.x() == pprev.x {
            if p.y() > pprev.y {
                punit = Point::new(p.x(), p.y() + hundred);
            } else {
                // implies p.y() < pprev.y()
                // it's safe not to explicitly cover p.y() == pprev.y() because that
                // implies that the x values are equal, and the y values are equal,
                // and this is impossible
                punit = Point::new(p.x(), p.y() - hundred);
            }
        } else {
            // implies p.x() < pprev.x()
            punit = Point::new(p.x(), p.y() + hundred);
        }
    } else if p.x() > pprev.x {
        punit = Point::new(p.x(), p.y() + hundred);
    } else if p.x() == pprev.x {
        if p.y() > pprev.y {
            punit = Point::new(p.x(), p.y() + hundred);
        } else {
            // implies p.y() < pprev.y()
            // it's safe not to explicitly cover p.y() == pprev.y() because that
            // implies that the x values are equal, and the y values are equal,
            // and this is impossible
            punit = Point::new(p.x(), p.y() - hundred);
        }
    } else {
        // implies p.x() < pprev.x()
        punit = Point::new(p.x(), p.y() - hundred);
    }
    let triarea = Triangle::from([p, punit, Point(pnext)]).signed_area();
    let edgelen = p.euclidean_distance(&Point(pnext));
    let mut sine = triarea / (T::from(0.5).unwrap() * T::from(100).unwrap() * edgelen);
    if sine < -T::one() || sine > T::one() {
        sine = T::one();
    }
    let angle;
    let perpunit = unitpvector(p, punit);
    let mut obtuse = false;
    let left = leftturn(p, perpunit, Point(pnext));
    if clockwise {
        if left == 0 {
            obtuse = true;
        }
        if left == -1 {
            angle = T::PI() / (T::one() + T::one());
        } else if !obtuse {
            angle = (-sine).asin();
        } else {
            angle = T::PI() - (-sine).asin();
        }
    } else {
        if left == 0 {
            obtuse = true;
        }
        if left == -1 {
            angle = T::PI() / (T::one() + T::one());
        } else if !obtuse {
            angle = sine.asin();
        } else {
            angle = T::PI() - sine.asin();
        }
    }
    angle
}

/// Does abc turn left?
fn leftturn<T>(a: Point<T>, b: Point<T>, c: Point<T>) -> i8
where
    T: Float,
{
    let narea = Triangle::from([a, b, c]).signed_area();
    if narea > T::zero() {
        1
    } else if narea < T::zero() {
        0
    } else {
        -1
    }
}

/// Calculate next set of caliper points
fn nextpoints<T>(state: &mut Polydist<T>)
where
    T: Float + FloatConst + Signed,
{
    state.alignment = None;
    state.ip1 = false;
    state.iq2 = false;
    state.ap1 = vertex_line_angle(
        state.poly1,
        state.p1,
        &state.slope,
        state.vertical,
        state.p1_idx,
    );
    state.aq2 = vertex_line_angle(
        state.poly2,
        state.q2,
        &state.slope,
        state.vertical,
        state.q2_idx,
    );
    let minangle = state.ap1.min(state.aq2);
    state.p1prev = state.p1;
    state.p1next = state.p1prev;
    state.q2prev = state.q2;
    state.q2next = state.q2prev;
    // iff (ap1 - minangle) is less than epsilon, alignment is edge-vertex (P-Q)
    // iff (aq2 - minangle) is less than epsilon, alignment is edge-vertex (Q-P)
    // if both are within epsilon, alignment is edge-edge
    // in each of the above, we also have to check for overlap, and in the case of
    // edge-edge alignment, additional cases must be considered.
    //
    // assume the calipers are rotated θ degrees around pi and qj, and that
    // we have hit vertex q` and edge [p`, p^]
    // check whether there exists a line segment [p, p*] which is orthogonal to [qj, q`]
    // compute the intersection of lines [pi, p*] and [qj, q`]
    // if this intersection q† exists, and ≠ qj or q`, compute the distance
    // between pi and q†, and compare it to the current minimum.
    // If the calipers intersect with edges on both polygons (implying the edges are parallel),
    // intersections must be computed between both segments, and if one is
    // found, the [pi, p`] - [qj, q`] edge-edge orthogonal distance is found and compared.
    // see Pirzadeh (1999), p31
    if (state.ap1 - minangle).abs() < T::from(0.002).unwrap() {
        state.ip1 = true;
        let p1next = next_vertex(state.poly1, state.p1_idx);
        state.p1next = Point(state.poly1.exterior().0[p1next]);
        state.p1_idx = p1next;
        state.alignment = Some(AlignedEdge::VertexP);
    }
    if (state.aq2 - minangle).abs() < T::from(0.002).unwrap() {
        state.iq2 = true;
        let q2next = next_vertex(state.poly2, state.q2_idx);
        state.q2next = Point(state.poly2.exterior().0[q2next]);
        state.q2_idx = q2next;
        state.alignment = match state.alignment {
            None => Some(AlignedEdge::VertexQ),
            Some(_) => Some(AlignedEdge::Edge),
        }
    }
    if state.ip1 {
        if state.p1.x() == state.p1next.x() {
            // The P line of support is vertical
            state.vertical = true;
            state.slope = T::zero();
        } else {
            state.vertical = false;
            state.slope = Line::new(state.p1next.0, state.p1.0).slope();
        }
    }
    if state.iq2 {
        if state.q2.x() == state.q2next.x() {
            // The Q line of support is vertical
            state.vertical = true;
            state.slope = T::zero();
        } else {
            state.vertical = false;
            state.slope = Line::new(state.q2next.0, state.q2.0).slope();
        }
    }
    // A start value's been set, and both polygon indices are in their initial
    // positions -- we're finished, so return the minimum distance
    if state.p1 == state.ymin1 && state.q2 == state.ymax2 && state.start.is_some() {
        state.finished = true;
    } else {
        state.start = Some(false);
        state.p1 = state.p1next;
        state.q2 = state.q2next;
    }
}

/// compute the minimum distance between entities (edges or vertices)
fn computemin<T>(state: &mut Polydist<T>)
where
    T: Float + Signed,
{
    let u;
    let u1;
    let u2;
    let mut newdist = state.p1.euclidean_distance(&state.q2);
    if newdist <= state.dist {
        // New minimum distance is between p1 and q2
        state.dist = newdist;
    }
    match state.alignment {
        Some(AlignedEdge::VertexP) => {
            // one line of support coincides with a vertex on Q, the other with an edge on P
            if !state.vertical {
                if state.slope != T::zero() {
                    u = unitvector(
                        &(-T::one() / state.slope),
                        state.poly2,
                        state.q2,
                        state.q2_idx,
                    );
                } else {
                    u = Point::new(state.q2.x(), state.q2.y() + T::from(100).unwrap());
                }
            } else {
                u = unitvector(&T::zero(), state.poly2, state.q2, state.q2_idx);
            }
            let line_1 = leftturn(u, state.q2, state.p1);
            let line_2 = leftturn(u, state.q2, state.p1prev);
            if line_1 != line_2 && line_1 != -1 && line_2 != -1 {
                // an orthogonal intersection exists
                newdist = vertex_line_distance(state.q2, state.p1prev, state.p1);
                if newdist <= state.dist {
                    // New minimum distance is between edge (p1prev, p1) and q2
                    state.dist = newdist;
                }
            }
        }
        Some(AlignedEdge::VertexQ) => {
            // one line of support coincides with a vertex on P, the other with an edge on Q
            if !state.vertical {
                if state.slope != T::zero() {
                    u = unitvector(
                        &(-T::one() / state.slope),
                        state.poly1,
                        state.p1,
                        state.p1_idx,
                    );
                } else {
                    u = Point::new(state.p1.x(), state.p1.y() + T::from(100).unwrap());
                }
            } else {
                u = unitvector(&T::zero(), state.poly1, state.p1, state.p1_idx);
            }
            let line_1 = leftturn(u, state.p1, state.q2);
            let line_2 = leftturn(u, state.p1, state.q2prev);
            if line_1 != line_2 && line_1 != -1 && line_2 != -1 {
                // an orthogonal intersection exists
                newdist = vertex_line_distance(state.p1, state.q2prev, state.q2);
                if newdist <= state.dist {
                    // New minimum distance is between edge (q2prev, q2) and p1
                    state.dist = newdist;
                }
            }
        }
        Some(AlignedEdge::Edge) => {
            // both lines of support coincide with edges (i.e. they're parallel)
            newdist = state.p1.euclidean_distance(&state.q2prev);
            if newdist <= state.dist {
                // New minimum distance is between p1 and q2prev
                state.dist = newdist;
            }
            newdist = state.p1prev.euclidean_distance(&state.q2);
            if newdist <= state.dist {
                // New minimum distance is between p1prev and q2
                state.dist = newdist;
            }
            if !state.vertical {
                if state.slope != T::zero() {
                    u1 = unitvector(
                        &(-T::one() / state.slope),
                        state.poly1,
                        state.p1prev,
                        state.p1_idx,
                    );
                    u2 = unitvector(
                        &(-T::one() / state.slope),
                        state.poly1,
                        state.p1,
                        state.p1_idx,
                    );
                } else {
                    u1 = Point::new(state.p1prev.x(), state.p1prev.y() + T::from(100).unwrap());
                    u2 = Point::new(state.p1.x(), state.p1.y() + T::from(100).unwrap());
                }
            } else {
                u1 = unitvector(&T::zero(), state.poly1, state.p1prev, state.p1_idx);
                u2 = unitvector(&T::zero(), state.poly1, state.p1, state.p1_idx);
            }
            let line_1a = leftturn(u1, state.p1prev, state.q2prev);
            let line_1b = leftturn(u1, state.p1prev, state.q2);
            let line_2a = leftturn(u2, state.p1, state.q2prev);
            let line_2b = leftturn(u2, state.p1, state.q2);
            if line_1a != line_1b && line_1a != -1 && line_1b != -1
                || line_2a != line_2b && line_2a != -1 && line_2b != -2
            {
                // an orthogonal intersection exists
                newdist = vertex_line_distance(state.p1, state.q2prev, state.q2);
                if newdist <= state.dist {
                    // New minimum distance is between edge (p1prev, p1) and q2prev
                    state.dist = newdist;
                }
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_vertex_line_distance() {
        let p = Point::new(0., 0.);
        let q = Point::new(3.8, 5.7);
        let r = Point::new(22.5, 10.);
        let dist = vertex_line_distance(p, q, r);
        assert_relative_eq!(dist, 6.850547423381579);
    }
}
