use num_traits::{Float, ToPrimitive};
use types::{Point, Line, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon};
use num_traits::float::FloatConst;
use algorithm::contains::Contains;
use algorithm::extremes::ExtremeIndices;
use algorithm::intersects::Intersects;
use algorithm::convexhull::ConvexHull;
use num_traits::pow::pow;

/// Returns the distance between two geometries.

pub trait Distance<T, Rhs = Self> {
    /// Returns the distance between two geometries
    ///
    /// If a `Point` is contained by a `Polygon`, the distance is `0.0`
    ///
    /// If a `Point` lies on a `Polygon`'s exterior or interior rings, the distance is `0.0`
    ///
    /// If a `Point` lies on a `LineString`, the distance is `0.0`
    ///
    /// The distance between a `Point` and an empty `LineString` is `0.0`
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point, LineString, Polygon};
    /// use geo::algorithm::distance::Distance;
    ///
    /// // Point to Point example
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance(&Point::new(-72.1260, 42.45));
    /// assert!(dist < COORD_PRECISION);
    ///
    /// // Point to Polygon example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    ///     (5., 1.)
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// let poly = Polygon::new(ls, vec![]);
    /// // A Random point outside the polygon
    /// let p = Point::new(2.5, 0.5);
    /// let dist = p.distance(&poly);
    /// assert_eq!(dist, 2.1213203435596424);
    ///
    /// // Point to LineString example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// // A Random point outside the LineString
    /// let p = Point::new(5.5, 2.1);
    /// let dist = p.distance(&ls);
    /// assert_eq!(dist, 1.1313708498984762);
    /// ```
    fn distance(&self, rhs: &Rhs) -> T;
}

// Return minimum distance between a Point and a Line segment
// This is a helper for Point-to-LineString and Point-to-Polygon distance
// adapted from https://github.com/OSGeo/geos/blob/master/src/algorithm/CGAlgorithms.cpp#L191
fn line_segment_distance<T>(point: &Point<T>, start: &Point<T>, end: &Point<T>) -> T
where
    T: Float + ToPrimitive,
{
    if start == end {
        return point.distance(start);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r = ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return point.distance(start);
    }
    if r >= T::one() {
        return point.distance(end);
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * (dx * dx + dy * dy).sqrt()
}

impl<T> Distance<T, Point<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance between two Points
    fn distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        dx.hypot(dy)
    }
}

impl<T> Distance<T, MultiPoint<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiPoint
    fn distance(&self, points: &MultiPoint<T>) -> T {
        points
            .0
            .iter()
            .map(|p| {
                let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
                dx.hypot(dy)
            })
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiPoint<T>
where
    T: Float,
{
    /// Minimum distance from a MultiPoint to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, Polygon<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a Polygon
    fn distance(&self, polygon: &Polygon<T>) -> T {
        // No need to continue if the polygon contains the point, or is zero-length
        if polygon.contains(self) || polygon.exterior.0.is_empty() {
            return T::zero();
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors
            .iter()
            .map(|ring| self.distance(ring))
            .fold(T::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior
                    .lines()
                    .map(|line| line_segment_distance(self, &line.start, &line.end))
                    .fold(T::max_value(), |accum, val| accum.min(val)),
            )
    }
}

impl<T> Distance<T, Point<T>> for Polygon<T>
where
    T: Float,
{
    /// Minimum distance from a Polygon to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, MultiPolygon<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiPolygon
    fn distance(&self, mpolygon: &MultiPolygon<T>) -> T {
        mpolygon
            .0
            .iter()
            .map(|p| self.distance(p))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiPolygon<T>
where
    T: Float,
{
    /// Minimum distance from a MultiPolygon to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, MultiLineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiLineString
    fn distance(&self, mls: &MultiLineString<T>) -> T {
        mls.0
            .iter()
            .map(|ls| self.distance(ls))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiLineString<T>
where
    T: Float,
{
    /// Minimum distance from a MultiLineString to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, LineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a LineString
    fn distance(&self, linestring: &LineString<T>) -> T {
        // No need to continue if the point is on the LineString, or it's empty
        if linestring.contains(self) || linestring.0.is_empty() {
            return T::zero();
        }
        linestring
            .lines()
            .map(|line| line_segment_distance(self, &line.start, &line.end))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for LineString<T>
where
    T: Float,
{
    /// Minimum distance from a LineString to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, Point<T>> for Line<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn distance(&self, point: &Point<T>) -> T {
        line_segment_distance(point, &self.start, &self.end)
    }
}
impl<T> Distance<T, Line<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn distance(&self, line: &Line<T>) -> T {
        line.distance(self)
// Polygon Distance
impl<T> Distance<T, Polygon<T>> for Polygon<T>
    where T: Float + FloatConst + Signed
{
    fn distance(&self, poly2: &Polygon<T>) -> T {
        if self.intersects(poly2) {
            return T::zero();
        }
        // TODO: check for containment
        min_poly_dist(&self.convex_hull(), &poly2.convex_hull())
    }
}

// calculate the minimum distance between two disjoint convex polygons
fn min_poly_dist<T>(poly1: &Polygon<T>, poly2: &Polygon<T>) -> T
    where T: Float + FloatConst + Signed
{
    let poly1_extremes = poly1.extreme_indices().unwrap();
    let poly2_extremes = poly2.extreme_indices().unwrap();
    let ymin1 = poly1.exterior.0[poly1_extremes.ymin];
    let ymax2 = poly2.exterior.0[poly2_extremes.ymax];
    // TODO: check whether the convex hulls intersect.
    // This means we'll have to use the Delaunay method from
    // http://www-cgrl.cs.mcgill.ca/~godfried/publications/mindist.pdf

    let mut state = Polydist {
        poly1: poly1,
        poly2: poly2,
        dist: T::infinity(),
        ymin1: ymin1,
        ymax2: ymax2,
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

#[derive(Debug)]
enum Aligned {
    EdgeVertexP,
    EdgeVertexQ,
    EdgeEdge,
}

// distance-finding state
#[derive(Debug)]
struct Polydist<'a, T>
    where T: Float,
          T: 'a
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
    alignment: Option<Aligned>,
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
    where T: Float
{
    fn next_vertex(&self, current_vertex: &usize) -> usize
        where T: Float
    {
        (current_vertex + 1) % (self.exterior.0.len() - 1)
    }
}

// Wrap-around previous-vertex
impl<T> Polygon<T>
    where T: Float
{
    fn prev_vertex(&self, current_vertex: &usize) -> usize
        where T: Float
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

// much of the following code is ported from Java, copyright 1999 Hormoz Pirzadeh, available at:
// http://web.archive.org/web/20150330010154/http://cgm.cs.mcgill.ca/%7Eorm/rotcal.html
fn unitvector<T>(slope: &T, poly: &Polygon<T>, p: &Point<T>, idx: &usize) -> Point<T>
    where T: Float
{
    let tansq = slope.powi(2);
    let cossq = T::one() / (T::one() + tansq);
    let sinsq = T::one() - cossq;
    let mut cos = T::zero();
    let mut sin;
    let pnext = poly.exterior.0[poly.next_vertex(idx)];
    let pprev = poly.exterior.0[poly.prev_vertex(idx)];
    let clockwise = if cross_prod(&pprev, p, &pnext) < T::zero() {
        true
    } else {
        false
    };
    let slope_prev;
    let slope_next;
    // Slope isn't 0, things are complicated
    if *slope != T::zero() {
        cos = cossq.sqrt();
        sin = sinsq.sqrt();
        if pnext.x() > p.x() {
            if pprev.x() > p.x() {
                if pprev.y() >= p.y() && pnext.y() >= p.y() {
                    if *slope > T::zero() {
                        slope_prev = (pprev.y() - p.y()) / (pprev.x() - p.x());
                        if clockwise && *slope <= slope_prev || !clockwise && *slope >= slope_prev {
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
                        slope_prev = (pprev.y() - p.y()) / (pprev.x() - p.x());
                        slope_next = (pnext.y() - p.y()) / (pnext.x() - p.x());
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
        } else if pnext.x() < p.x() {
            if pprev.x() < p.x() {
                if pprev.y() >= p.y() && pnext.y() >= p.y() {
                    if *slope > T::zero() {
                        if clockwise {
                            cos = -cos;
                            sin = -sin;
                        }
                    } else {
                        slope_prev = (p.y() - pprev.y()) / (p.x() - pprev.x());
                        slope_next = (p.y() - pnext.y()) / (p.x() - pnext.x());
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
                } else if pprev.y() <= p.y() && pnext.y() <= p.y() {
                    if *slope > T::zero() {
                        slope_next = (p.y() - pnext.y()) / (p.x() - pnext.x());
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
    where T: Float
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
fn vertex_line_angle<T>(poly: &Polygon<T>, p: &Point<T>, m: &T, vertical: bool, idx: &usize) -> T
    where T: Float + FloatConst
{
    let hundred = T::from(100).unwrap();
    let pnext = poly.exterior.0[poly.next_vertex(idx)];
    let pprev = poly.exterior.0[poly.prev_vertex(idx)];
    let clockwise = if cross_prod(&pprev, p, &pnext) < T::zero() {
        true
    } else {
        false
    };
    let punit;
    if !vertical {
        punit = unitvector(m, poly, p, idx);
    } else {
        match clockwise {
            true => {
                if p.x() > pprev.x() {
                    punit = Point::new(p.x(), p.y() - hundred);
                } else if p.x() == pprev.x() {
                    if p.y() > pprev.y() {
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
            }
            false => {
                if p.x() > pprev.x() {
                    punit = Point::new(p.x(), p.y() + hundred);
                } else if p.x() == pprev.x() {
                    if p.y() > pprev.y() {
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
            }
        }
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

// positive implies a -> b -> c is counter-clockwise, negative implies clockwise
pub fn cross_prod<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> T
    where T: Float
{
    (p_b.x() - p_a.x()) * (p_c.y() - p_a.y()) - (p_b.y() - p_a.y()) * (p_c.x() - p_a.x())
}

// Does abc turn left?
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
    where T: Float + FloatConst
{
    state.alignment = None;
    state.ip1 = false;
    state.iq2 = false;
    state.ap1 = vertex_line_angle(state.poly1,
                                  &state.p1,
                                  &state.slope,
                                  state.vertical,
                                  &state.p1_idx);
    state.aq2 = vertex_line_angle(state.poly2,
                                  &state.q2,
                                  &state.slope,
                                  state.vertical,
                                  &state.q2_idx);
    let minangle = state.ap1.min(state.aq2);
    state.p1prev = state.p1;
    state.p1next = state.p1prev;
    state.q2prev = state.q2;
    state.q2next = state.q2prev;
    // iff (ap1 - minangle) is less than epsilon, alignment is edge-vertex (P-Q)
    // iff (aq2 - minangle) is less than epsilon, alignment is edge-vertex (Q-P)
    // if both are within epsilon, alignment is edge-edge, and we need to check for overlap
    // overlap is defined by the possibility of drawing an orthogonal line
    // between the two edges at any points other than their vertices
    // see Pirzadeh (1999), p31
    if (state.ap1 - minangle).abs() < T::from(0.002).unwrap() {
        state.ip1 = true;
        let p1next = state.poly1.next_vertex(&state.p1_idx);
        state.p1next = state.poly1.exterior.0[p1next];
        state.p1_idx = p1next;
        state.alignment = Some(Aligned::EdgeVertexP);
    }
    if (state.aq2 - minangle).abs() < T::from(0.002).unwrap() {
        state.iq2 = true;
        let q2next = state.poly2.next_vertex(&state.q2_idx);
        state.q2next = state.poly2.exterior.0[q2next];
        state.q2_idx = q2next;
        state.alignment = match state.alignment {
            None => Some(Aligned::EdgeVertexQ),
            Some(_) => Some(Aligned::EdgeEdge),
        }
    }
    if state.ip1 {
        match state.p1.x() == state.p1next.x() {
            // The P line of support is vertical
            true => {
                state.vertical = true;
                state.slope = T::zero();
            }
            false => {
                state.vertical = false;
                if state.p1.x() > state.p1next.x() {
                    state.slope = (state.p1.y() - state.p1next.y()) /
                                  (state.p1.x() - state.p1next.x());
                } else {
                    state.slope = (state.p1next.y() - state.p1.y()) /
                                  (state.p1next.x() - state.p1.x());
                }
            }
        }
    }
    if state.iq2 {
        match state.q2.x() == state.q2next.x() {
            true => {
                // The Q line of support is vertical
                state.vertical = true;
                state.slope = T::zero();
            }
            false => {
                state.vertical = false;
                if state.q2.x() > state.q2next.x() {
                    state.slope = (state.q2.y() - state.q2next.y()) /
                                  (state.q2.x() - state.q2next.x());
                } else {
                    state.slope = (state.q2next.y() - state.q2.y()) /
                                  (state.q2next.x() - state.q2.x());
                }
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

// compute the minimum distance between entities (edges or vertices)
fn computemin<T>(state: &mut Polydist<T>)
    where T: Float
{
    let u;
    let u1;
    let u2;
    let mut newdist = state.p1.distance(&state.q2);
    if newdist <= state.dist {
        // New minimum distance is between p1 and q2
        state.dist = newdist;
    }
    match state.alignment {
        Some(Aligned::EdgeVertexP) => {
            // one line of support coincides with a vertex on Q, the other with an edge on P
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
                    // New minimum distance is between edge (p1prev, p1) and q2
                    state.dist = newdist;
                }
            }
        }
        Some(Aligned::EdgeVertexQ) => {
            // one line of support coincides with a vertex on P, the other with an edge on Q
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
                    // New minimum distance is between edge (q2prev, q2) and p1
                    state.dist = newdist;
                }
            }
        }
        Some(Aligned::EdgeEdge) => {
            // both lines of support coincide with edges (i.e. they're parallel)
            // we need to check for overlap
            newdist = state.p1.distance(&state.q2prev);
            if newdist <= state.dist {
                // New minimum distance is between p1 and q2prev
                state.dist = newdist;
            }
            newdist = state.p1prev.distance(&state.q2);
            if newdist <= state.dist {
                // New minimum distance is between p1prev and q2
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
    use types::{Point, Line, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon};
    use algorithm::distance::{Distance, line_segment_distance};
    use algorithm::convexhull::ConvexHull;
    use super::*;

    #[test]
    fn line_segment_distance_test() {
        let o1 = Point::new(8.0, 0.0);
        let o2 = Point::new(5.5, 0.0);
        let o3 = Point::new(5.0, 0.0);
        let o4 = Point::new(4.5, 1.5);

        let p1 = Point::new(7.2, 2.0);
        let p2 = Point::new(6.0, 1.0);

        let dist = line_segment_distance(&o1, &p1, &p2);
        let dist2 = line_segment_distance(&o2, &p1, &p2);
        let dist3 = line_segment_distance(&o3, &p1, &p2);
        let dist4 = line_segment_distance(&o4, &p1, &p2);
        // Results agree with Shapely
        assert_relative_eq!(dist, 2.0485900789263356);
        assert_relative_eq!(dist2, 1.118033988749895);
        assert_relative_eq!(dist3, 1.4142135623730951);
        assert_relative_eq!(dist4, 1.5811388300841898);
        // Point is on the line
        let zero_dist = line_segment_distance(&p1, &p1, &p2);
        assert_relative_eq!(zero_dist, 0.0);
    }
    #[test]
    // Point to Polygon, outside point
    fn point_polygon_distance_outside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point outside the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 2.1213203435596424);
    }
    #[test]
    // Point to Polygon, inside point
    fn point_polygon_distance_inside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn point_polygon_distance_boundary_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn flibble() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 0.0004),
            Point::new(0.0004, 0.0004),
            Point::new(0.0004, 0.),
            Point::new(0., 0.),
        ]);

        let poly = Polygon::new(exterior.clone(), vec![]);
        let bugged_point = Point::new(0.0001, 0.);
        assert_eq!(poly.distance(&bugged_point), 0.);
    }
    #[test]
    // Point to Polygon, empty Polygon
    fn point_polygon_empty_test() {
        // an empty Polygon
        let points = vec![];
        let ls = LineString(points);
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon with an interior ring
    fn point_polygon_interior_cutout_test() {
        // an octagon
        let ext_points = vec![
            (4., 1.),
            (5., 2.),
            (5., 3.),
            (4., 4.),
            (3., 4.),
            (2., 3.),
            (2., 2.),
            (3., 1.),
            (4., 1.),
        ];
        // cut out a triangle inside octagon
        let int_points = vec![(3.5, 3.5), (4.4, 1.5), (2.6, 1.5), (3.5, 3.5)];
        let ls_ext = LineString(ext_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let ls_int = LineString(int_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // A point inside the cutout triangle
        let p = Point::new(3.5, 2.5);
        let dist = p.distance(&poly);
        // 0.41036467732879783 <-- Shapely
        assert_relative_eq!(dist, 0.41036467732879767);
    }
    #[test]
    fn point_distance_multipolygon_test() {
        let ls1 = LineString(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 10.0),
            Point::new(2.0, 0.0),
            Point::new(0.0, 0.0),
        ]);
        let ls2 = LineString(vec![
            Point::new(3.0, 0.0),
            Point::new(4.0, 10.0),
            Point::new(5.0, 0.0),
            Point::new(3.0, 0.0),
        ]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mp = MultiPolygon(vec![p1, p2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.distance(&mp), 60.959002616512684);
    }
    #[test]
    // Point to LineString
    fn point_linestring_distance_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A Random point "inside" the LineString
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 1.1313708498984762);
    }
    #[test]
    // Point to LineString, point lies on the LineString
    fn point_linestring_contains_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A point which lies on the LineString
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to LineString, closed triangle
    fn point_linestring_triangle_test() {
        let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let p = Point::new(3.5, 2.5);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.5);
    }
    #[test]
    // Point to LineString, empty LineString
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    fn distance_multilinestring_test() {
        let v1 = LineString(vec![Point::new(0.0, 0.0), Point::new(1.0, 10.0)]);
        let v2 = LineString(vec![
            Point::new(1.0, 10.0),
            Point::new(2.0, 0.0),
            Point::new(3.0, 1.0),
        ]);
        let mls = MultiLineString(vec![v1, v2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.distance(&mls), 63.25345840347388);
    }
    #[test]
    fn distance1_test() {
        assert_eq!(
            Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)),
            1.
        );
    }
    #[test]
    fn distance2_test() {
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert_relative_eq!(dist, 146.99163308930207);
    }
    #[test]
    fn distance_multipoint_test() {
        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let mp = MultiPoint(v);
        let p = Point::new(50.0, 50.0);
        assert_eq!(p.distance(&mp), 64.03124237432849)
    }
    #[test]
    // test edge-vertex minimum distance
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
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 21.0);
    }
    #[test]
    // test vertex-vertex minimum distance
    fn test_minimum_polygon_distance_2() {
        let points_raw = vec![(118., 200.), (153., 179.), (106., 155.), (88., 190.), (118., 200.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 =
            vec![(242., 186.), (260., 146.), (182., 175.), (216., 193.), (242., 186.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 29.274562336608895);
    }
    #[test]
    // test edge-edge minimum distance
    fn test_minimum_polygon_distance_3() {
        let points_raw = vec![(182., 182.), (182., 168.), (138., 160.), (136., 193.), (182., 182.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 =
            vec![(232., 196.), (234., 150.), (194., 165.), (194., 191.), (232., 196.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 12.0);
    }
    #[test]
    fn test_vertex_line_distance() {
        let p = Point::new(0., 0.);
        let q = Point::new(3.8, 5.7);
        let r = Point::new(22.5, 10.);
        let dist = p.vertex_line_distance(&q, &r);
        assert_eq!(dist, 6.850547423381579);
    }
}
