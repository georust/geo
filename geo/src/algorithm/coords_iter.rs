use crate::{coord, Coord, CoordNum};
use geo_traits::{
    to_geo::ToGeoCoord, CoordTrait, GeometryCollectionTrait, GeometryTrait, GeometryType,
    LineStringTrait, LineTrait, MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait,
    PointTrait, PolygonTrait, RectTrait, TriangleTrait,
};
use std::marker::PhantomData; // Added import


/// Iterate over geometry coordinates.
pub trait CoordsIter {
    type T: CoordNum;

    type Iter<'a>: Iterator<Item = Coord<Self::T>>
    where
        Self: 'a,
        Self::T: 'a + CoordNum;

    type ExteriorIter<'a>: Iterator<Item = Coord<Self::T>>
    where
        Self: 'a,
        Self::T: 'a + CoordNum;

    /// Iterate over all exterior and (if any) interior coordinates of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    ///
    /// let multi_point = geo::MultiPoint::new(vec![
    ///     geo::point!(x: -10., y: 0.),
    ///     geo::point!(x: 20., y: 20.),
    ///     geo::point!(x: 30., y: 40.),
    /// ]);
    ///
    /// let mut iter = multi_point.coords_iter();
    /// assert_eq!(Some(geo::coord! { x: -10., y: 0. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 20., y: 20. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 30., y: 40. }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn coords_iter(&self) -> Self::Iter<'_>;

    /// Return the number of coordinates in a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    /// use geo::line_string;
    ///
    /// let ls = line_string![
    ///     (x: 1., y: 2.),
    ///     (x: 23., y: 82.),
    ///     (x: -1., y: 0.),
    /// ];
    ///
    /// assert_eq!(3, ls.coords_count());
    /// ```
    fn coords_count(&self) -> usize;

    /// Iterate over all exterior coordinates of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    /// use geo::polygon;
    ///
    /// // a diamond shape
    /// let polygon = polygon![
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// let mut iter = polygon.exterior_coords_iter();
    /// assert_eq!(Some(geo::coord! { x: 1., y: 0. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 2., y: 1. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 1., y: 2. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 0., y: 1. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 1., y: 0. }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_>;
}

// Helper function for Polygon coordinate count
fn polygon_coords_count<T, P>(polygon: &P) -> usize
where
    T: CoordNum,
    P: PolygonTrait<T = T>,
    // Bounds required by the function body (no lifetimes needed for count)
    // P::RingType is implicitly bound via P: PolygonTrait
{
    let exterior_count = polygon
        .exterior()
        .map(|ls| LineStringTrait::num_coords(&ls))
        .unwrap_or(0);
    let interior_count = polygon
        .interiors()
        .map(|ls| LineStringTrait::num_coords(&ls))
        .sum::<usize>();
    exterior_count + interior_count
}

impl<G> CoordsIter for G
where
    G: GeometryTrait,
    G::T: CoordNum,
{
    type T = G::T;

    type Iter<'a>
        = Iter<'a, G>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = ExteriorIter<'a, G>
    where
        Self: Sized + 'a,
        Self::T: 'a + CoordNum;

    fn coords_iter(&self) -> Self::Iter<'_> {
        match self.as_type() {
            GeometryType::Point(p) => Iter::Point(Some(PointTrait::coord(p).unwrap().to_coord())),
            GeometryType::Line(l) => Iter::Line(LineIter { line: l, index: 0 }),
            GeometryType::LineString(ls) => Iter::LineString(LineStringIter {
                linestring: ls,
                index: 0,
            }),
            GeometryType::Polygon(p) => Iter::Polygon(PolygonIter::new(p)),
            GeometryType::MultiPoint(mp) => Iter::MultiPoint(MultiPointIter {
                multi_point: mp,
                index: 0,
            }),
            GeometryType::MultiLineString(mls) => Iter::MultiLineString(MultiLineStringIter::new(mls)),
            GeometryType::MultiPolygon(mp) => {
                Iter::MultiPolygon(MultiPolygonIter::new(mp))
                // Iter::MultiPolygon(MapCoordsIter(mp.polygons(), marker::PhantomData).flatten())
            },
            GeometryType::GeometryCollection(gc) => {
                // Boxing is likely necessary here due to heterogeneous nature
                // and complexity of tracking state across different geometry types
                // without significant code complexity or allocations anyway.
                let mut all_coords = Vec::new();
                for g in GeometryCollectionTrait::geometries(gc) {
                    all_coords.extend(g.coords_iter());
                }
                Iter::GeometryCollection(Box::new(all_coords.into_iter()))
            }
            GeometryType::Rect(r) => Iter::Rect(RectIter::new(r)),
            GeometryType::Triangle(t) => Iter::Triangle(TriangleIter::new(t)),
        }
    }

    fn coords_count(&self) -> usize {
        match self.as_type() {
            GeometryType::Point(_) => 1,
            GeometryType::Line(_) => 2,
            GeometryType::LineString(ls) => LineStringTrait::num_coords(ls),
            GeometryType::Polygon(p) => polygon_coords_count(p),
            GeometryType::MultiPoint(mp) => MultiPointTrait::num_points(mp),
            GeometryType::MultiLineString(mls) => MultiLineStringTrait::line_strings(mls)
                .map(|ls| LineStringTrait::num_coords(&ls))
                .sum::<usize>(),
            GeometryType::MultiPolygon(mp) => MultiPolygonTrait::polygons(mp)
                .map(|p| polygon_coords_count(&p))
                .sum::<usize>(), // Pass ref to helper
            GeometryType::GeometryCollection(gc) => GeometryCollectionTrait::geometries(gc)
                .map(|g| g.coords_count())
                .sum::<usize>(),
            GeometryType::Rect(_r) => 5,     // 4 corners + closing coord
            GeometryType::Triangle(_t) => 4, // 3 corners + closing coord
        }
    }

    fn exterior_coords_iter(&self) -> ExteriorIter<G> {
        match self.as_type() {
            GeometryType::Point(p) => ExteriorIter::Point(Some(PointTrait::coord(p).unwrap().to_coord())),
            GeometryType::Line(l) => ExteriorIter::Line(LineIter { line: l, index: 0 }),
            GeometryType::LineString(ls) => ExteriorIter::LineString(LineStringIter {
                linestring: ls,
                index: 0,
            }),
            GeometryType::Polygon(p) => ExteriorIter::Polygon(PolygonExteriorIter {
                polygon: p,
                index: 0,
            }),
            GeometryType::MultiPoint(mp) => ExteriorIter::MultiPoint(MultiPointIter {
                multi_point: mp,
                index: 0,
            }),
            GeometryType::MultiLineString(mls) => {
                ExteriorIter::MultiLineString(MultiLineStringIter::new(mls))
            }
            GeometryType::MultiPolygon(mp) => {
                ExteriorIter::MultiPolygon(MultiPolygonExteriorIter::new(mp))
            }
            GeometryType::GeometryCollection(gc) => {
                // For GeometryCollection, we need to collect into a Vec first
                // because GeometryTrait is not dyn-compatible
                let mut all_exterior_coords = Vec::new();
                for g in GeometryCollectionTrait::geometries(gc) {
                    all_exterior_coords.extend(g.exterior_coords_iter());
                }
                ExteriorIter::GeometryCollection(Box::new(all_exterior_coords.into_iter()))
            }
            GeometryType::Rect(r) => ExteriorIter::Rect(RectIter::new(r)),
            GeometryType::Triangle(t) => ExteriorIter::Triangle(TriangleIter::new(t)),
        }
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

/// Helper iterator for Line coordinates
pub struct LineIter<'a, L: LineTrait + 'a>
where
    L::T: CoordNum,
{
    line: &'a L,
    index: usize, // 0 for start, 1 for end
}

impl<'a, L> Iterator for LineIter<'a, L>
where
    L: LineTrait + 'a,
    L::T: CoordNum,
{
    type Item = Coord<L::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            0 => {
                self.index += 1;
                Some(self.line.start().to_coord())
            }
            1 => {
                self.index += 1;
                Some(self.line.end().to_coord())
            }
            _ => None,
        }
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

/// Helper iterator for LineString coordinates
pub struct LineStringIter<'a, L: LineStringTrait + 'a>
where
    L::T: CoordNum,
{
    linestring: &'a L,
    index: usize,
}

impl<'a, L> Iterator for LineStringIter<'a, L>
where
    L: LineStringTrait + 'a,
    L::T: CoordNum,
{
    type Item = Coord<L::T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if the current index is within bounds
        if self.index < LineStringTrait::num_coords(self.linestring) {
            // Get the coordinate at the current index and convert it
            let coord = self.linestring.coord(self.index)?;

            // Increment the index for the next call
            self.index += 1;

            Some(coord.to_coord())
        } else {
            None
        }
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘

/// State for the PolygonIter
enum PolygonIterState {
    Exterior,
    Interior(usize), // Holds the current interior ring index
    Done,
}

/// Helper iterator for Polygon coordinates (exterior + interiors)
pub struct PolygonIter<'a, P: PolygonTrait + 'a>
where
    P::T: CoordNum,
{
    polygon: &'a P,
    state: PolygonIterState,
    /// Current coordinate index within the current ring
    coord_index: usize,
}

impl<'a, P> PolygonIter<'a, P>
where
    P: PolygonTrait + 'a,
    P::T: CoordNum,
{
    fn new(polygon: &'a P) -> Self {
        let initial_state = if polygon.exterior().is_some() {
            PolygonIterState::Exterior
        } else if polygon.interior(0).is_some() {
             PolygonIterState::Interior(0)
        } else {
            PolygonIterState::Done
        };
        Self {
            polygon,
            state: initial_state,
            coord_index: 0,
        }
    }
}

impl<'a, P> Iterator for PolygonIter<'a, P>
where
    P: PolygonTrait + 'a,
    P::T: CoordNum,
{
    type Item = Coord<P::T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                PolygonIterState::Exterior => {
                    if let Some(ring) = self.polygon.exterior() {
                        if let Some(coord) = ring.coord(self.coord_index) {
                            // Got coord from exterior
                            self.coord_index += 1;
                            return Some(coord.to_coord());
                        } else {
                            // Finished exterior, move to first interior
                            self.state = PolygonIterState::Interior(0);
                            self.coord_index = 0;
                            // Continue loop to process interior 0
                        }
                    } else {
                        // No exterior, move to first interior
                        self.state = PolygonIterState::Interior(0);
                        self.coord_index = 0;
                        // Continue loop to process interior 0
                    }
                }
                PolygonIterState::Interior(ring_idx) => {
                    if let Some(ring) = self.polygon.interior(ring_idx) {
                        if let Some(coord) = ring.coord(self.coord_index) {
                            // Got coord from interior ring
                            self.coord_index += 1;
                            return Some(coord.to_coord());
                        } else {
                            // Finished this interior ring, move to next
                            self.state = PolygonIterState::Interior(ring_idx + 1);
                            self.coord_index = 0;
                            // Continue loop to process next interior
                        }
                    } else {
                        // No more interior rings
                        self.state = PolygonIterState::Done;
                        return None;
                    }
                }
                PolygonIterState::Done => {
                    return None;
                }
            }
        }
    }
}

pub struct PolygonExteriorIter<'a, P: PolygonTrait> {
    polygon: &'a P,
    index: usize,
}

impl<'a, P> Iterator for PolygonExteriorIter<'a, P>
where
    P: PolygonTrait + 'a,
    P::T: CoordNum,
{
    type Item = Coord<P::T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the exterior ring
        let exterior = self.polygon.exterior()?;

        // Check if the current index is within bounds
        if self.index < LineStringTrait::num_coords(&exterior) {
            // Get the coordinate at the current index and convert it
            let coord = exterior.coord(self.index)?;

            // Increment the index for the next call
            self.index += 1;

            Some(coord.to_coord())
        } else {
            None
        }
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

/// Helper iterator for MultiPoint coordinates
pub struct MultiPointIter<'a, MP: MultiPointTrait + 'a>
where
    MP::T: CoordNum,
{
    multi_point: &'a MP,
    index: usize,
}

impl<'a, MP> Iterator for MultiPointIter<'a, MP>
where
    MP: MultiPointTrait + 'a,
    MP::T: CoordNum,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < MultiPointTrait::num_points(self.multi_point) {
            let point = self.multi_point.point(self.index)?;
            self.index += 1;
            // Extract coord first to satisfy borrow checker
            let coord = PointTrait::coord(&point).unwrap().to_coord();
            Some(coord)
        } else {
            None
        }
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

/// Helper iterator for MultiLineString coordinates
pub struct MultiLineStringIter<'a, MLS: MultiLineStringTrait + 'a>
where
    MLS::T: CoordNum,
{
    multi_linestring: &'a MLS,
    /// Current LineString index
    ls_index: usize,
    /// Current coordinate index within the current LineString
    coord_index: usize,
}

impl<'a, MLS> MultiLineStringIter<'a, MLS>
where
    MLS: MultiLineStringTrait + 'a,
    MLS::T: CoordNum,
{
    fn new(multi_linestring: &'a MLS) -> Self {
        Self {
            multi_linestring,
            ls_index: 0,
            coord_index: 0,
        }
    }
}

impl<'a, MLS> Iterator for MultiLineStringIter<'a, MLS>
where
    MLS: MultiLineStringTrait + 'a,
    MLS::T: CoordNum,
{
    type Item = Coord<MLS::T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the current LineString, if the index is valid
        let current_ls = self.multi_linestring.line_string(self.ls_index)?;

        // Try to get the coordinate from the current LineString
        let x = match current_ls.coord(self.coord_index) {
            Some(coord) => {
                // Found coordinate, advance index for next call
                self.coord_index += 1;
                Some(coord.to_coord())
            }
            None => {
                // Reached end of current LineString, move to the next one
                self.ls_index += 1;
                self.coord_index = 0;
                self.next() // Recursively call next to get coord from the new LineString
            }
        }; x
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

/// State for the MultiPolygonIter
#[derive(Clone, Copy, Debug)]
enum MultiPolygonIterState {
    Exterior { poly_idx: usize },
    Interior { poly_idx: usize, ring_idx: usize },
    Done,
}

/// Helper iterator for MultiPolygon coordinates
pub struct MultiPolygonIter<'a, MP: MultiPolygonTrait + 'a>
where
    MP::T: CoordNum,
{
    multi_polygon: &'a MP,
    state: MultiPolygonIterState,
    /// Current coordinate index within the current ring
    coord_index: usize,
}


impl<'a, MP> MultiPolygonIter<'a, MP>
where
    MP: MultiPolygonTrait + 'a,
    MP::T: CoordNum,
{
    fn new(multi_polygon: &'a MP) -> Self {
        Self {
            multi_polygon,
            // Find the initial state by searching for the first valid ring
            state: Self::find_first_state(multi_polygon, 0),
            coord_index: 0,
        }
    }

    // Helper to find the initial state (first valid ring) starting from poly_idx 0
    fn find_first_state(multi_polygon: &'a MP, start_poly_idx: usize) -> MultiPolygonIterState {
        for poly_idx in start_poly_idx..multi_polygon.num_polygons() {
            if let Some(poly) = multi_polygon.polygon(poly_idx) {
                if poly.exterior().is_some() {
                    return MultiPolygonIterState::Exterior { poly_idx };
                }
                if poly.interior(0).is_some() {
                    return MultiPolygonIterState::Interior { poly_idx, ring_idx: 0 };
                }
                // If polygon has no rings, continue to the next polygon
            }
        }
        // No valid rings found in any polygon
        MultiPolygonIterState::Done
    }

    // Helper to find the next state after finishing a ring
    fn find_next_state(&self, current_state: MultiPolygonIterState) -> MultiPolygonIterState {
        match current_state {
            MultiPolygonIterState::Exterior { poly_idx } => {
                // Finished exterior, try first interior of the same polygon
                if let Some(poly) = self.multi_polygon.polygon(poly_idx) {
                    if poly.interior(0).is_some() {
                        return MultiPolygonIterState::Interior { poly_idx, ring_idx: 0 };
                    }
                }
                // No interiors, or polygon doesn't exist (shouldn't happen here),
                // try next polygon's exterior
                Self::find_first_state(self.multi_polygon, poly_idx + 1)
            }
            MultiPolygonIterState::Interior { poly_idx, ring_idx } => {
                // Finished an interior, try next interior of the same polygon
                 if let Some(poly) = self.multi_polygon.polygon(poly_idx) {
                    if poly.interior(ring_idx + 1).is_some() {
                        return MultiPolygonIterState::Interior { poly_idx, ring_idx: ring_idx + 1 };
                    }
                }
                // No more interiors, or polygon doesn't exist (shouldn't happen here),
                // try next polygon's exterior
                Self::find_first_state(self.multi_polygon, poly_idx + 1)
            }
            MultiPolygonIterState::Done => MultiPolygonIterState::Done,
        }
    }
} // Added closing brace for impl MultiPolygonIter

/// Helper iterator for MultiPolygon exterior coordinates (all exterior rings)
pub struct MultiPolygonExteriorIter<'a, MP: MultiPolygonTrait + 'a>
where
    MP::T: CoordNum,
{
    multi_polygon: &'a MP,
    /// Current Polygon index
    poly_index: usize,
    /// Current coordinate index within the current polygon's exterior ring
    coord_index: usize,
    /// Phantom data to hold the coordinate type
    _phantom: PhantomData<MP::T>,
}

impl<'a, MP> MultiPolygonExteriorIter<'a, MP>
where
    MP: MultiPolygonTrait + 'a,
    MP::T: CoordNum,
{
    fn new(multi_polygon: &'a MP) -> Self {
        Self {
            multi_polygon,
            poly_index: 0,
            coord_index: 0,
            _phantom: PhantomData,
        }
    }
}


impl<'a, MP> Iterator for MultiPolygonExteriorIter<'a, MP>
where
    MP: MultiPolygonTrait + 'a,
    MP::T: CoordNum,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the current polygon, or return None if we've iterated past the end
        let poly = match self.multi_polygon.polygon(self.poly_index) {
            Some(p) => p,
            None => return None,
        };

        // Get the exterior ring, or advance to the next polygon if it doesn't exist
        let ring = match poly.exterior() {
            Some(r) => r,
            None => {
                self.poly_index += 1;
                self.coord_index = 0;
                return self.next(); // Recursively call to process the next polygon
            }
        };

        // Get the coordinate from the ring, or advance to the next polygon if we're at the end of the ring
        let x = match ring.coord(self.coord_index) {
            Some(coord) => {
                // Found coordinate, advance index and return
                self.coord_index += 1;
                Some(coord.to_coord())
            }
            None => {
                // Reached end of this ring, move to the next polygon
                self.poly_index += 1;
                self.coord_index = 0;
                self.next() // Recursively call to process the next polygon
            }
        }; x
    }
}

impl<'a, MP> Iterator for MultiPolygonIter<'a, MP>
where
    MP: MultiPolygonTrait + 'a,
    MP::T: CoordNum,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_state = self.state; // Copy state to avoid borrow issues in find_next_state
// Get the polygon index from the current state, or return None if done
let poly_idx = match current_state {
    MultiPolygonIterState::Exterior { poly_idx } => poly_idx,
    MultiPolygonIterState::Interior { poly_idx, .. } => poly_idx,
    MultiPolygonIterState::Done => return None,
};

// Get the polygon itself (must exist if state is not Done)
let poly = self.multi_polygon.polygon(poly_idx).unwrap();

// Get the relevant ring based on the state
let ring_opt = match current_state {
    MultiPolygonIterState::Exterior { .. } => poly.exterior(),
    MultiPolygonIterState::Interior { ring_idx, .. } => poly.interior(ring_idx),
    MultiPolygonIterState::Done => unreachable!(), // Already handled
};

        // Ring must exist if we are in Exterior/Interior state
        let ring = ring_opt.unwrap();

        // Try to get the coordinate
        let x = match ring.coord(self.coord_index) {
            Some(coord) => {
                // Found coordinate, advance index and return
                self.coord_index += 1;
                Some(coord.to_coord())
            }
            None => {
                // Reached end of this ring, find the next state and recurse
                self.state = self.find_next_state(current_state);
                self.coord_index = 0;
                self.next()
            }
        }; x
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

/// Helper iterator for Rect coordinates
pub struct RectIter<T: CoordNum> {
    coords: [Coord<T>; 5],
    index: usize,
}

impl<T: CoordNum> RectIter<T> {
    fn new<R: RectTrait<T = T>>(rect: &R) -> Self {
        let min_coord = rect.min();
        let max_coord = rect.max();
        let min_x = min_coord.x();
        let min_y = min_coord.y();
        let max_x = max_coord.x();
        let max_y = max_coord.y();
        let coords = [
            coord! { x: min_x, y: min_y }, // Bottom-left
            coord! { x: max_x, y: min_y }, // Bottom-right
            coord! { x: max_x, y: max_y }, // Top-right
            coord! { x: min_x, y: max_y }, // Top-left
            coord! { x: min_x, y: min_y }, // Close the ring
        ];
        Self { coords, index: 0 }
    }
}

impl<T: CoordNum> Iterator for RectIter<T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 5 {
            let coord = self.coords[self.index];
            self.index += 1;
            Some(coord)
        } else {
            None
        }
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

/// Helper iterator for Triangle coordinates
pub struct TriangleIter<T: CoordNum> {
    coords: [Coord<T>; 4],
    index: usize,
}

impl<T: CoordNum> TriangleIter<T> {
    fn new<Tr: TriangleTrait<T = T>>(triangle: &Tr) -> Self {
        let c1 = triangle.first();
        let c2 = triangle.second();
        let c3 = triangle.third();
        let coords = [
            coord! { x: c1.x(), y: c1.y() },
            coord! { x: c2.x(), y: c2.y() },
            coord! { x: c3.x(), y: c3.y() },
            coord! { x: c1.x(), y: c1.y() }, // Close the ring
        ];
        Self { coords, index: 0 }
    }
}

impl<T: CoordNum> Iterator for TriangleIter<T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 4 {
            let coord = self.coords[self.index];
            self.index += 1;
            Some(coord)
        } else {
            None
        }
    }
}

// ┌─────────────────────┐
// │ Main Iterator Enums │
// └─────────────────────┘

pub enum Iter<'a, G: GeometryTrait + 'a>
where
    G::T: CoordNum,
{
    /// An iterator for a single point. Option is used to implement Iterator easily.
    Point(Option<Coord<G::T>>),
    /// An iterator for a Line's coordinates.
    Line(LineIter<'a, G::LineType<'a>>),
    /// An iterator for a LineString's coordinates.
    LineString(LineStringIter<'a, G::LineStringType<'a>>),
    /// An iterator for a Polygon's coordinates.
    Polygon(PolygonIter<'a, G::PolygonType<'a>>),
    /// An iterator for a MultiPoint's coordinates.
    MultiPoint(MultiPointIter<'a, G::MultiPointType<'a>>),
    /// An iterator for a MultiLineString's coordinates.
    MultiLineString(MultiLineStringIter<'a, G::MultiLineStringType<'a>>),
    /// An iterator for a MultiPolygon's coordinates.
    MultiPolygon(MultiPolygonIter<'a, G::MultiPolygonType<'a>>),
    /// An iterator for a Rect's coordinates.
    Rect(RectIter<G::T>),
    /// An iterator for a Triangle's coordinates.
    Triangle(TriangleIter<G::T>),
    /// Boxed iterator for GeometryCollection
    GeometryCollection(Box<dyn Iterator<Item = Coord<G::T>> + 'a>),
}

impl<'a, G> Iterator for Iter<'a, G>
where
    G: GeometryTrait + 'a,
    G::T: CoordNum,
{
    type Item = Coord<G::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Point(coord_opt) => coord_opt.take(),
            Iter::Line(iter) => iter.next(),
            Iter::LineString(iter) => iter.next(),
            Iter::Polygon(iter) => iter.next(),
            Iter::MultiPoint(iter) => iter.next(),
            Iter::MultiLineString(iter) => iter.next(),
            Iter::MultiPolygon(iter) => iter.next(),
            Iter::Rect(iter) => iter.next(),
            Iter::Triangle(iter) => iter.next(),
            Iter::GeometryCollection(iter) => iter.next(),
        }
    }
}

#[derive(Default)]
pub enum ExteriorIter<'a, G: GeometryTrait + 'a>
where
    G::T: CoordNum,
{
    #[default]
    Empty,
    /// Iterator for a single point's exterior (which is just the point itself)
    Point(Option<Coord<G::T>>),
    /// Iterator for a Line's exterior (its start and end points)
    Line(LineIter<'a, G::LineType<'a>>),
    /// Iterator for a LineString's exterior (all its points)
    LineString(LineStringIter<'a, G::LineStringType<'a>>),
    /// Iterator for a Polygon's exterior ring
    Polygon(PolygonExteriorIter<'a, G::PolygonType<'a>>),
    /// Iterator for a MultiPoint's exterior (all its points)
    MultiPoint(MultiPointIter<'a, G::MultiPointType<'a>>),
    /// Iterator for a MultiLineString's exterior (all its points)
    MultiLineString(MultiLineStringIter<'a, G::MultiLineStringType<'a>>),
    /// Iterator for a MultiPolygon's exterior rings
    MultiPolygon(MultiPolygonExteriorIter<'a, G::MultiPolygonType<'a>>),
    /// Iterator for a Rect's exterior ring
    Rect(RectIter<G::T>),
    /// Iterator for a Triangle's exterior ring
    Triangle(TriangleIter<G::T>),
    /// Boxed iterator for complex types (GeometryCollection)
    GeometryCollection(Box<dyn Iterator<Item = Coord<G::T>> + 'a>),
}

impl<'a, G> Iterator for ExteriorIter<'a, G>
where
    G: GeometryTrait + 'a,
    G::T: CoordNum,
{
    type Item = Coord<G::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ExteriorIter::Empty => None,
            ExteriorIter::Point(coord_opt) => coord_opt.take(),
            ExteriorIter::Line(iter) => iter.next(),
            ExteriorIter::LineString(iter) => iter.next(),
            ExteriorIter::Polygon(iter) => iter.next(),
            ExteriorIter::MultiPoint(iter) => iter.next(),
            ExteriorIter::MultiLineString(iter) => iter.next(), // Added
            ExteriorIter::MultiPolygon(iter) => iter.next(),    // Added
            ExteriorIter::Rect(iter) => iter.next(),
            ExteriorIter::Triangle(iter) => iter.next(),
            ExteriorIter::GeometryCollection(iter) => iter.next(), // Keep for GeometryCollection
        }
    }
}

// ┌───────────────────┐
// │ Utility Iterators │
// └───────────────────┘

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
struct MapCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter<T = T>,
>(Iter1, std::marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter<T = T>> Iterator
    for MapCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::Iter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[cfg(test)]
mod test {
    use super::CoordsIter;
    use crate::{
        coord, line_string, point, polygon, Coord, Geometry, GeometryCollection, Line, LineString,
        MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
    };

    #[test]
    fn test_point() {
        let (point, expected_coords) = create_point();
        let actual_coords = point.coords_iter().collect::<Vec<_>>();
        assert_eq!(expected_coords, actual_coords);
        assert_eq!(point.coords_count(), 1);
    }

    #[test]
    fn test_line() {
        let line = Line::new(coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. });
        let coords = line.coords_iter().collect::<Vec<_>>();
        assert_eq!(
            vec![coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. },],
            coords
        );
        assert_eq!(line.coords_count(), 2);
    }

    #[test]
    fn test_line_string() {
        let (line_string, expected_coords) = create_line_string();
        let count = expected_coords.len();
        let actual_coords = line_string.coords_iter().collect::<Vec<_>>();
        assert_eq!(expected_coords, actual_coords);
        assert_eq!(line_string.coords_count(), count);
    }

    #[test]
    fn test_polygon() {
        let (polygon, expected_coords) = create_polygon();
        let count = expected_coords.len();
        let actual_coords = polygon.coords_iter().collect::<Vec<_>>();
        assert_eq!(expected_coords, actual_coords);
        assert_eq!(polygon.coords_count(), count);

        // Test exterior iter
        let exterior_coords = polygon.exterior_coords_iter().collect::<Vec<_>>();
        assert_eq!(
            exterior_coords,
            vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 5.0, y: 10.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
            ]
        );
    }

    #[test]
    fn test_multi_point() {
        let point1 = point!(x: 1., y: 2.);
        let point2 = point!(x: 3., y: 4.);
        let expected_coords = vec![coord! { x: 1., y: 2. }, coord! { x: 3., y: 4. }];

        let multi_point = MultiPoint::new(vec![point1, point2]);
        let actual_coords = multi_point.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
        assert_eq!(multi_point.coords_count(), 2);
    }

    #[test]
    fn test_multi_line_string() {
        let line_string1 = line_string![(x: 1., y: 2.), (x: 2., y: 3.)];
        let line_string2 = line_string![(x: 10., y: 20.), (x: 30., y: 40.), (x: 50., y: 60.)];
        let expected_coords = vec![
            coord! { x: 1., y: 2. },
            coord! { x: 2., y: 3. },
            coord! { x: 10., y: 20. },
            coord! { x: 30., y: 40. },
            coord! { x: 50., y: 60. },
        ];
        let count1 = line_string1.coords_count();
        let count2 = line_string2.coords_count();

        let multi_line_string = MultiLineString::new(vec![line_string1, line_string2]);
        let actual_coords = multi_line_string.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
        assert_eq!(multi_line_string.coords_count(), count1 + count2);
    }

    #[test]
    fn test_multi_polygon() {
        let (polygon1, _) = create_polygon();
        let polygon2 = polygon!(exterior: [(x: 10., y: 10.), (x: 15., y: 20.), (x: 20., y: 10.), (x: 10., y: 10.)], interiors: []);
        let count1 = polygon1.coords_count();
        let count2 = polygon2.coords_count();
        let mut expected_coords = polygon1.coords_iter().collect::<Vec<_>>();
        expected_coords.extend(polygon2.coords_iter());

        let multi_polygon = MultiPolygon::new(vec![polygon1.clone(), polygon2.clone()]);
        let actual_coords = multi_polygon.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
        assert_eq!(multi_polygon.coords_count(), count1 + count2);

        // Test exterior iter
        let exterior_coords = multi_polygon.exterior_coords_iter().collect::<Vec<_>>();
        let expected_exterior = multi_polygon.0[0]
            .exterior_coords_iter()
            .chain(multi_polygon.0[1].exterior_coords_iter())
            .collect::<Vec<_>>();
        assert_eq!(exterior_coords, expected_exterior);
    }

    #[test]
    fn test_geometry() {
        let (line_string, expected_coords) = create_line_string();
        let count = expected_coords.len();
        let geometry = Geometry::LineString(line_string);
        let actual_coords = geometry.coords_iter().collect::<Vec<_>>();
        assert_eq!(expected_coords, actual_coords);
        assert_eq!(geometry.coords_count(), count);
    }

    #[test]
    fn test_rect() {
        let (rect, expected_coords) = create_rect();
        let count = expected_coords.len();
        let actual_coords = rect.coords_iter().collect::<Vec<_>>();

        assert_eq!(rect.coords_count(), count);
        assert_eq!(actual_coords.len(), count);
        // Sort vectors for comparison as f64 cannot be hashed/Eq
        let mut actual_sorted = actual_coords;
        actual_sorted.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then_with(|| a.y.partial_cmp(&b.y).unwrap())
        });
        let mut expected_sorted = expected_coords;
        expected_sorted.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then_with(|| a.y.partial_cmp(&b.y).unwrap())
        });
        assert_eq!(actual_sorted, expected_sorted);
    }

    #[test]
    fn test_triangle() {
        let (triangle, expected_coords) = create_triangle();
        let count = expected_coords.len();
        let actual_coords = triangle.coords_iter().collect::<Vec<_>>();
        assert_eq!(triangle.coords_count(), count);
        assert_eq!(actual_coords.len(), count);
        // Sort vectors for comparison as f64 cannot be hashed/Eq
        let mut actual_sorted = actual_coords;
        actual_sorted.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then_with(|| a.y.partial_cmp(&b.y).unwrap())
        });
        let mut expected_sorted = expected_coords;
        expected_sorted.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap()
                .then_with(|| a.y.partial_cmp(&b.y).unwrap())
        });
        assert_eq!(actual_sorted, expected_sorted);
    }

    #[test]
    fn test_geometry_collection() {
        let (line_string, _) = create_line_string();
        let (polygon, _) = create_polygon();
        let ls_count = line_string.coords_count();
        let poly_count = polygon.coords_count();
        let mut expected_coords = line_string.coords_iter().collect::<Vec<_>>();
        expected_coords.extend(polygon.coords_iter());

        let collection = GeometryCollection::new_from(vec![
            Geometry::LineString(line_string.clone()),
            Geometry::Polygon(polygon.clone()),
        ]);
        let actual_coords = collection.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
        assert_eq!(collection.coords_count(), ls_count + poly_count);

        // Test exterior iter
        let exterior_coords = collection.exterior_coords_iter().collect::<Vec<_>>();
        let expected_exterior = line_string
            .exterior_coords_iter()
            .chain(polygon.exterior_coords_iter())
            .collect::<Vec<_>>();
        assert_eq!(exterior_coords, expected_exterior);
    }

    #[test]
    #[ignore = "CoordsIter impl for arrays/slices removed; requires GeometryTrait impl for them"]
    fn test_array() {
        let _coords = [
            coord! { x: 1., y: 2. },
            coord! { x: 3., y: 4. },
            coord! { x: 5., y: 6. },
        ];
        // let actual_coords = coords.coords_iter().collect::<Vec<_>>();
        // assert_eq!(coords.to_vec(), actual_coords);
    }

    #[test]
    #[ignore = "CoordsIter impl for arrays/slices removed; requires GeometryTrait impl for them"]
    fn test_slice() {
        let _coords = &[
            coord! { x: 1., y: 2. },
            coord! { x: 3., y: 4. },
            coord! { x: 5., y: 6. },
        ];
        // let actual_coords = coords.coords_iter().collect::<Vec<_>>();
        // assert_eq!(coords.to_vec(), actual_coords);
    }

    // Helper functions for creating test geometries

    fn create_point() -> (Point, Vec<Coord>) {
        (point!(x: 1., y: 2.), vec![coord! { x: 1., y: 2. }])
    }

    fn create_triangle() -> (Triangle, Vec<Coord>) {
        (
            Triangle::new(
                coord! { x: 1., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 5., y: 6. },
            ),
            // Triangle exterior includes closing coordinate
            vec![
                coord! { x: 1., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 5., y: 6. },
                coord! { x: 1., y: 2. },
            ],
        )
    }

    fn create_rect() -> (Rect, Vec<Coord>) {
        (
            Rect::new(coord! { x: 1., y: 2. }, coord! { x: 3., y: 4. }),
            // Rect exterior includes closing coordinate
            vec![
                coord! { x: 1., y: 2. }, // bottom-left
                coord! { x: 3., y: 2. }, // bottom-right
                coord! { x: 3., y: 4. }, // top-right
                coord! { x: 1., y: 4. }, // top-left
                coord! { x: 1., y: 2. }, // closing coordinate
            ],
        )
    }

    fn create_line_string() -> (LineString, Vec<Coord>) {
        (
            line_string![
                (x: 1., y: 2.),
                (x: 2., y: 3.),
            ],
            vec![coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. }],
        )
    }

    fn create_polygon() -> (Polygon<f64>, Vec<Coord>) {
        (
            polygon!(
                exterior: [(x: 0., y: 0.), (x: 5., y: 10.), (x: 10., y: 0.), (x: 0., y: 0.)],
                interiors: [[(x: 1., y: 1.), (x: 9., y: 1.), (x: 5., y: 9.), (x: 1., y: 1.)]],
            ),
            vec![
                // Exterior
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 5.0, y: 10.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                // Interior
                coord! { x: 1.0, y: 1.0 },
                coord! { x: 9.0, y: 1.0 },
                coord! { x: 5.0, y: 9.0 },
                coord! { x: 1.0, y: 1.0 },
            ],
        )
    }
}
