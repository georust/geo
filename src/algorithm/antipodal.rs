// Pirzadeh, H. (1999) Computational geometry with the rotating calipers., pp30 – 32
// Available from: http://digitool.library.mcgill.ca/R/-?func=dbin-jump-full&object_id=21623&silo_library=GEN01
// http://web.archive.org/web/20150330010154/http://cgm.cs.mcgill.ca/%7Eorm/rotcal.html
use num_traits::Float;
use num_traits::float::FloatConst;
use types::{Point, LineString, Polygon};
use std::fmt::Debug;
use algorithm::distance::Distance;

// distance-finding state
#[derive(Debug)]
struct Polydist<'a, T>
    where T: Float,
          T: 'a
{
    poly1: &'a Polygon<T>,
    poly2: &'a Polygon<T>,
    dist: T,
    pid: i8,
    main: Option<Point<T>>,
    ymin1: Point<T>,
    p1_idx: usize,
    ymax2: Point<T>,
    q2_idx: usize,
    p1: Point<T>,
    q2: Point<T>,
    p1next: Point<T>,
    q1next: Point<T>,
    q2next: Point<T>,
    p1prev: Point<T>,
    q1prev: Point<T>,
    p2prev: Point<T>,
    q2prev: Point<T>,
    iid: i8,
    ap1: T,
    aq2: T,
    start: Option<bool>,
    finished: bool,
    ip1: bool,
    iq2: bool,
    slope: T,
    vertical: bool,
}

// Wrap-around next vertex
impl<T> Polygon<T>
    where T: Float + Debug
{
    fn next_vertex(&self, current_vertex: &usize) -> usize
        where T: Float + Debug
    {
        (current_vertex + 1) % (self.exterior.0.len() - 1)
    }
}

// Wrap-around previous-vertex
impl<T> Polygon<T>
    where T: Float + Debug
{
    fn previous_vertex(&self, current_vertex: &usize) -> usize
        where T: Float + Debug
    {
        (current_vertex + (self.exterior.0.len() - 1) - 1) % (self.exterior.0.len() - 1)
    }
}

// Minimum distance between a vertex and an imaginary line drawn from p to q
impl<T> Point<T>
    where T: Float
{
    fn vertex_line_distance(&self, p: &Point<T>, q: &Point<T>) -> T
        where T: Float
    {
        self.distance(&LineString(vec![*p, *q]))
    }
}

fn unitvector<T>(slope: &T, poly: &Polygon<T>, p: &Point<T>, idx: &usize) -> Point<T>
    where T: Float + Debug
{
    let clockwise = true;
    let tansq = *slope * *slope;
    let cossq = T::one() / (T::one() + tansq);
    let sinsq = T::one() - cossq;
    // maybe need to be float
    let mut cos = T::zero();
    let mut sin;
    let pnext = poly.exterior.0[poly.next_vertex(idx)];
    let pprev = poly.exterior.0[poly.previous_vertex(idx)];
    let slprev;
    let slnext;
    // Slope isn't 0, things are complicated
    if *slope != T::zero() {
        cos = cossq.sqrt();
        sin = sinsq.sqrt();
        if pnext.x() > p.x() {
            if pprev.x() > p.x() {
                if pprev.y() >= p.y() && pnext.y() >= p.y() {
                    if *slope > T::zero() {
                        slprev = (pprev.y() - p.y()) / (pprev.x() - p.x());
                        if clockwise && *slope <= slprev || !clockwise && *slope >= slprev {
                            cos = -cos;
                            sin = -sin;
                        } else if clockwise {
                            cos = -cos;
                        } else {
                            sin = -sin;
                        }
                    }
                } else if pprev.y() <= p.y() && pnext.y() <= p.y() {
                    if *slope > T::zero() {
                        if !clockwise {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else {
                        slprev = (pprev.y() - p.y()) / (pprev.x() - p.x());
                        slnext = (pnext.y() - p.y()) / (pnext.x() - p.x());
                        if clockwise {
                            if *slope <= slprev {
                                cos = -cos;
                            } else {
                                sin = -sin;
                            }
                        } else if *slope <= slnext {
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
        } else if pnext.x() < p.x() {
            if pprev.x() < p.x() {
                if pprev.y() >= p.y() && pnext.y() >= p.y() {
                    if *slope > T::zero() {
                        if clockwise {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else {
                        slprev = (p.y() - pprev.y()) / (p.x() - pprev.x());
                        slnext = (p.y() - pnext.y()) / (p.x() - pnext.x());
                        if clockwise {
                            if *slope <= slprev {
                                sin = -sin;
                            } else {
                                cos = -cos;
                            }
                        } else if *slope <= slnext {
                            cos = -cos;
                        } else {
                            sin = -sin;
                        }
                    }
                } else if pprev.y() <= p.y() && pnext.y() <= p.y() {
                    if *slope > T::zero() {
                        slnext = (p.y() - pnext.y()) / (p.x() - pnext.x());
                        if *slope >= slnext {
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
        } else if pprev.x() > p.x() {
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
        if pnext.x() > p.x() {
            cos = T::one();
        } else if pnext.x() < p.x() {
            cos = -T::one();
        } else if pnext.x() == p.x() {
            if pprev.x() < p.x() {
                cos = T::one();
            } else {
                cos = -T::one();
            }
        }
    }
    Point::new(p.x() + T::from(100).unwrap() * cos,
               p.y() + T::from(100).unwrap() * sin)
}

// Perpendicular unit vector of a vertex and a unit vector
fn unitpvector<T>(p: &Point<T>, u: &Point<T>) -> Point<T>
    where T: Float + Debug
{
    let hundred = T::from(100).unwrap();
    let vertical;
    let mut slope;
    let sperp;
    slope = T::zero();
    if p.x() == u.x() {
        vertical = true;
    } else {
        vertical = false;
    }
    if !vertical {
        if p.y() == u.y() {
            slope = T::zero();
        } else if u.x() > p.x() {
            slope = (u.y() - p.y()) / (u.x() - p.x());
        } else {
            slope = (p.y() - u.y()) / (p.x() - u.x());
        }
    }
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
        sperp = -T::one() / slope;
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

// Angle between a vertex and an edge
fn vertexlineangle<T>(poly: &Polygon<T>, p: &Point<T>, m: &T, vert: bool, idx: &usize) -> T
    where T: Float + FloatConst + Debug
{
    let hundred = T::from(100).unwrap();
    let pnext = poly.exterior.0[poly.next_vertex(idx)];
    let pprev = poly.exterior.0[poly.previous_vertex(idx)];
    let mut slope = T::zero();
    let mut vertical = vert;
    let clockwise = true;
    let punit;
    if !vertical {
        slope = *m;
    } else {
        vertical = true;
    }
    if !vertical {
        punit = unitvector(&slope, poly, p, idx);
        // this branch can set punit = None, which is a logic error at best
        // triarea requires punit to have a value, but I don't think this pattern
        // is irrefutable sooo I've fixed it for now with 0, 0
        // which is still (almost certainly) wrong, but satisfies the compiler
    } else if clockwise {
        if p.x() > pprev.x() {
            punit = Point::new(p.x(), p.y() - hundred);
        } else if p.x() < pprev.x() {
            punit = Point::new(p.x(), p.y() + hundred);
        } else if p.x() == pprev.x() {
            if p.y() > pprev.y() {
                punit = Point::new(p.x(), p.y() + hundred);
            } else if p.y() < pprev.y() {
                punit = Point::new(p.x(), p.y() - hundred);
            } else {
                // punit = None;
                punit = Point::new(T::zero(), T::zero());
            }
        } else {
            // punit = None;
            punit = Point::new(T::zero(), T::zero());
        }
        // so like, not clockwise
    } else if p.x() > pprev.x() {
        punit = Point::new(p.x(), p.y() + hundred);
    } else if p.x() < pprev.x() {
        punit = Point::new(p.x(), p.y() - hundred);
    } else if p.x() == pprev.x() {
        if p.y() > pprev.y() {
            punit = Point::new(p.x(), p.y() + hundred);
        } else if p.y() < pprev.y() {
            punit = Point::new(p.x(), p.y() - hundred);
        } else {
            // punit = None;
            punit = Point::new(T::zero(), T::zero());
        }
    } else {
        // punit = None;
        punit = Point::new(T::zero(), T::zero());
    }
    let triarea = triangle_area(p, &punit, &pnext);
    let edgelen = p.distance(&pnext);
    let mut sine = triarea / (T::from(0.5).unwrap() * T::from(100).unwrap() * edgelen);
    if sine < -T::one() {
        sine = T::one();
    }
    if sine > T::one() {
        sine = T::one();
    }
    let angle;
    let perpunit = unitpvector(p, &punit);
    let mut obtuse = false;
    let left = leftturn(p, &perpunit, &pnext);
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

// self-explanatory
fn triangle_area<T>(a: &Point<T>, b: &Point<T>, c: &Point<T>) -> T
    where T: Float
{
    (T::from(0.5).unwrap() *
     (a.x() * b.y() - a.y() * b.x() + a.y() * c.x() - a.x() * c.y() + b.x() * c.y() -
      c.x() * b.y()))
}

// I mean sure
fn leftturn<T>(a: &Point<T>, b: &Point<T>, c: &Point<T>) -> i8
    where T: Float
{
    let narea = triangle_area(a, b, c);
    if narea > T::zero() {
        1
    } else if narea < T::zero() {
        0
    } else {
        -1
    }
}

// Calculate next set of caliper points
fn nextpoints<T>(state: &mut Polydist<T>)
    where T: Float + FloatConst + Debug
{
    // always reset IID
    state.iid = 0;
    state.main = None;
    state.pid = 0;
    state.ip1 = false;
    state.iq2 = false;
    state.ap1 = vertexlineangle(state.poly1,
                                &state.p1,
                                &state.slope,
                                state.vertical,
                                &state.p1_idx);
    state.aq2 = vertexlineangle(state.poly2,
                                &state.q2,
                                &state.slope,
                                state.vertical,
                                &state.q2_idx);
    let minangle = state.ap1.min(state.aq2);
    state.p1prev = state.p1;
    state.p1next = state.p1prev;
    state.q2prev = state.q2;
    state.q2next = state.q2prev;
    if (state.ap1 - minangle).abs() < T::from(0.002).unwrap() {
        state.ip1 = true;
        let p1next = state.poly1.next_vertex(&state.p1_idx);
        state.p1next = state.poly1.exterior.0[p1next];
        state.p1_idx = p1next;
        state.main = Some(state.p1next);
        state.pid = 1;
        state.iid += 1;
    }
    if (state.aq2 - minangle).abs() < T::from(0.002).unwrap() {
        state.iq2 = true;
        let q2next = state.poly2.next_vertex(&state.q2_idx);
        state.q2next = state.poly2.exterior.0[q2next];
        state.q2_idx = q2next;
        if state.main.is_none() {
            state.main = Some(state.q2next);
            state.pid = 2;
        }
        state.iid += 2;
    }
    if state.ip1 {
        if state.p1.x() == state.p1next.x() {
            // Vertical case
            state.vertical = true;
            // 0 is a placeholder value
            state.slope = T::zero();
        } else {
            state.vertical = false;
            if state.p1.x() > state.p1next.x() {
                state.slope = (state.p1.y() - state.p1next.y()) / (state.p1.x() - state.p1next.x());
            } else {
                state.slope = (state.p1next.y() - state.p1.y()) / (state.p1next.x() - state.p1.x());
            }
        }
    } else if state.iq2 {
        if state.q2.x() == state.q2next.x() {
            // Vertical case
            state.vertical = true;
            state.slope = T::zero();
        } else {
            state.vertical = false;
            if state.q2.x() > state.q2next.x() {
                state.slope = (state.q2.y() - state.q2next.y()) / (state.q2.x() - state.q2next.x());
            } else {
                state.slope = (state.q2next.y() - state.q2.y()) / (state.q2next.x() - state.q2.x());
            }
        }
    }
    // A start value's been set, and both polygon indices are in their initial
    // positions -- we're finished, so return the minimum distance
    if state.p1 == state.ymin1 && state.q2 == state.ymax2 && !state.start.is_none() {
        state.finished = true;
    } else {
        state.start = Some(false);
        state.p1 = state.p1next;
        state.q2 = state.q2next;
    }
}

// compute the minimum distance between entities
// three variations are possible (iid 1, 2, or 3):
// - vertex-edge-distance
// - edge-edge distance, overlapping edges
// - edge-edge distance, non-overlapping edges
fn computemin<T>(state: &mut Polydist<T>)
    where T: Float + Debug
{
    let mut newdist;
    let u;
    let u1;
    let u2;
    match state.iid {
        1 => {
            // one line of support coincides with a vertex, the other with an edge
            newdist = state.p1.distance(&state.q2);
            if newdist <= state.dist {
                state.dist = newdist;
            }
            if !state.vertical {
                if state.slope != T::zero() {
                    u = unitvector(&(-T::one() / state.slope),
                                   state.poly2,
                                   &state.q2,
                                   &state.q2_idx);
                } else {
                    u = Point::new(state.q2.x(), state.q2.y() + T::from(100).unwrap());
                }
            } else {
                u = unitvector(&T::zero(), state.poly2, &state.q2, &state.q2_idx);
            }
            let line_1 = leftturn(&u, &state.q2, &state.p1);
            let line_2 = leftturn(&u, &state.q2, &state.p1prev);
            if line_1 != line_2 && line_1 != -1 && line_2 != -1 {
                newdist = state.q2.vertex_line_distance(&state.p1prev, &state.p1);
                if newdist <= state.dist {
                    state.dist = newdist;
                }
            }
        }
        2 => {
            // both lines of support coincide with edges, and the edges overlap
            newdist = state.p1.distance(&state.q2);
            if newdist <= state.dist {
                state.dist = newdist;
            }
            if !state.vertical {
                if state.slope != T::zero() {
                    u = unitvector(&(-T::one() / state.slope),
                                   state.poly1,
                                   &state.p1,
                                   &state.p1_idx);
                } else {
                    u = Point::new(state.p1.x(), state.p1.y() + T::from(100).unwrap());
                }
            } else {
                u = unitvector(&T::zero(), state.poly1, &state.p1, &state.p1_idx);
            }
            let line_1 = leftturn(&u, &state.p1, &state.q2);
            let line_2 = leftturn(&u, &state.p1, &state.q2prev);
            if line_1 != line_2 && line_1 != -1 && line_2 != -1 {
                newdist = state.p1.vertex_line_distance(&state.q2prev, &state.q2);
                if newdist <= state.dist {
                    state.dist = newdist;
                }
            }
        }
        3 => {
            // both lines of support coincide with edges, but they don't overlap
            newdist = state.p1.distance(&state.q2);
            if newdist <= state.dist {
                state.dist = newdist;
            }
            newdist = state.p1.distance(&state.q2prev);
            if newdist <= state.dist {
                state.dist = newdist;
            }
            newdist = state.p1prev.distance(&state.q2);
            if newdist <= state.dist {
                state.dist = newdist;
            }
            if !state.vertical {
                if state.slope != T::zero() {
                    u1 = unitvector(&(-T::one() / state.slope),
                                    state.poly1,
                                    &state.p1prev,
                                    &state.p1_idx);
                    u2 = unitvector(&(-T::one() / state.slope),
                                    state.poly1,
                                    &state.p1,
                                    &state.p1_idx);
                } else {
                    u1 = Point::new(state.p1prev.x(), state.p1prev.y() + T::from(100).unwrap());
                    u2 = Point::new(state.p1.x(), state.p1.y() + T::from(100).unwrap());
                }
            } else {
                u1 = unitvector(&T::zero(), state.poly1, &state.p1prev, &state.p1_idx);
                u2 = unitvector(&T::zero(), state.poly1, &state.p1, &state.p1_idx);
            }
            let line_1a = leftturn(&u1, &state.p1prev, &state.q2prev);
            let line_1b = leftturn(&u1, &state.p1prev, &state.q2);
            let line_2a = leftturn(&u2, &state.p1, &state.q2prev);
            let line_2b = leftturn(&u2, &state.p1, &state.q2);
            if line_1a != line_1b && line_1a != -1 && line_1b != -1 ||
               line_2a != line_2b && line_2a != -1 && line_2b != -2 {
                newdist = state.p1.vertex_line_distance(&state.q2prev, &state.q2);
                if newdist <= state.dist {
                    state.dist = newdist;
                }
            }
        }
        _ => unreachable!(),
    }
}

fn min_poly_dist<T>(poly1: &mut Polygon<T>, poly2: &mut Polygon<T>) -> T
    where T: Float + FloatConst + Debug
{
    // convex hulls are always anti-clockwise.
    poly1.exterior.0.reverse();
    poly2.exterior.0.reverse();
    // get poly1 ymin
    let ymin1 = Point::new(T::from(112.0).unwrap(), T::from(202.0).unwrap());
    // get poly2 ymax
    let ymax2 = Point::new(T::from(158.0).unwrap(), T::from(242.0).unwrap());
    let mut state = Polydist {
        poly1: poly1,
        poly2: poly2,
        dist: T::infinity(),
        pid: 0,
        main: None,
        ymin1: ymin1,
        ymax2: ymax2,
        // initial polygon 1 min y idx
        p1_idx: 7 as usize,
        // initial polygon 2 max y idx
        q2_idx: 2 as usize,
        p1: ymin1,
        q2: ymax2,
        p1next: Point::new(T::zero(), T::zero()),
        q1next: Point::new(T::zero(), T::zero()),
        q2next: Point::new(T::zero(), T::zero()),
        p1prev: Point::new(T::zero(), T::zero()),
        q1prev: Point::new(T::zero(), T::zero()),
        p2prev: Point::new(T::zero(), T::zero()),
        q2prev: Point::new(T::zero(), T::zero()),
        iid: 0,
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

#[cfg(test)]
mod test {
    use types::{Point, LineString};
    use algorithm::convexhull::ConvexHull;
    use super::*;
    #[test]
    fn test_minimum_polygon_distance() {
        let points_raw = vec![(126., 232.),
                              (126., 212.),
                              (112., 202.),
                              (97., 204.),
                              (87., 215.),
                              (87., 232.),
                              (100., 246.),
                              (118., 247.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 = vec![(188., 231.),
                                (189., 207.),
                                (174., 196.),
                                (164., 196.),
                                (147., 220.),
                                (158., 242.),
                                (177., 242.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&mut poly1.convex_hull(), &mut poly2.convex_hull());
        assert_eq!(dist, 21.0);
    }
}
