use std::borrow::Borrow;
use std::fmt::Debug;
use std::option;

use geo_traits::*;
use geo_traits_ext::*;

use crate::geometry::*;
use crate::{coord, CoordNum};

use std::{iter, marker, slice};

type CoordinateChainOnce<T> = iter::Chain<iter::Once<Coord<T>>, iter::Once<Coord<T>>>;

/// Iterate over geometry coordinates.
pub trait CoordsIter {
    type Iter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;
    type ExteriorIter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;
    type Scalar: CoordNum;

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

pub trait CoordsIterTrait<GT: GeoTypeTag> {
    type Scalar: CoordNum;

    type Iter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;

    type ExteriorIter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;

    fn coords_iter_trait(&self) -> Self::Iter<'_>;

    fn coords_count_trait(&self) -> usize;

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_>;
}

// ┌──────────────────────────┐
// │ Implementation for Point │
// └──────────────────────────┘

impl<T: CoordNum> CoordsIter for Point<T> {
    type Iter<'a>
        = iter::Once<Coord<T>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
    }

    /// Return the number of coordinates in the `Point`.
    fn coords_count(&self) -> usize {
        1
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

impl<T, P> CoordsIterTrait<PointTag> for P
where
    T: CoordNum,
    P: PointTraitExt<T = T>,
{
    type Iter<'a>
        = option::IntoIter<Coord<T>>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        self.geo_coord().into_iter()
    }

    fn coords_count_trait(&self) -> usize {
        self.coord().map_or(0, |_| 1)
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        self.geo_coord().into_iter()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

impl<T: CoordNum> CoordsIter for Line<T> {
    type Iter<'a>
        = iter::Chain<iter::Once<Coord<T>>, iter::Once<Coord<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.start).chain(iter::once(self.end))
    }

    /// Return the number of coordinates in the `Line`.
    fn coords_count(&self) -> usize {
        2
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

impl<T, L> CoordsIterTrait<LineTag> for L
where
    T: CoordNum,
    L: LineTraitExt<T = T>,
{
    type Iter<'a>
        = iter::Chain<iter::Once<Coord<T>>, iter::Once<Coord<T>>>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        iter::once(self.start_coord()).chain(iter::once(self.end_coord()))
    }

    fn coords_count_trait(&self) -> usize {
        2
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter_trait()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

type LineStringIter<'a, T> = iter::Copied<slice::Iter<'a, Coord<T>>>;

impl<T: CoordNum> CoordsIter for LineString<T> {
    type Iter<'a>
        = LineStringIter<'a, T>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.0.iter().copied()
    }

    /// Return the number of coordinates in the `LineString`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

pub struct LineStringCoordIter<LS, LSB>
where
    LS: LineStringTraitExt<T: CoordNum>,
    LSB: Borrow<LS>,
{
    ls: Option<LSB>,
    idx: usize,
    limit: usize,
    _marker: marker::PhantomData<LS>,
}

impl<LS, LSB> LineStringCoordIter<LS, LSB>
where
    LS: LineStringTraitExt<T: CoordNum>,
    LSB: Borrow<LS>,
{
    fn new(ls_opt: Option<LSB>) -> Self {
        match &ls_opt {
            Some(ls) => {
                let limit = ls.borrow().num_coords();
                Self {
                    ls: ls_opt,
                    idx: 0,
                    limit,
                    _marker: marker::PhantomData,
                }
            }
            None => Self {
                ls: None,
                idx: 0,
                limit: 0,
                _marker: marker::PhantomData,
            },
        }
    }
}

impl<LS, LSB> Iterator for LineStringCoordIter<LS, LSB>
where
    LS: LineStringTraitExt<T: CoordNum>,
    LSB: Borrow<LS>,
{
    type Item = Coord<LS::T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.limit {
            None
        } else {
            let coord = unsafe {
                // unwrap should be safe here. If ls is None, limit is 0, and we would not reach here.
                // We also have self.idx < self.limit, so we are not accessing out of bounds.
                self.ls
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .geo_coord_unchecked(self.idx)
            };
            self.idx += 1;
            Some(coord)
        }
    }
}

impl<T, LS> CoordsIterTrait<LineStringTag> for LS
where
    T: CoordNum,
    LS: LineStringTraitExt<T = T>,
{
    type Iter<'a>
        = LineStringCoordIter<LS, &'a LS>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        LineStringCoordIter::new(Some(self))
    }

    fn coords_count_trait(&self) -> usize {
        self.num_coords()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter_trait()
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘

type PolygonIter<'a, T> = iter::Chain<
    LineStringIter<'a, T>,
    iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>,
>;

impl<T: CoordNum> CoordsIter for Polygon<T> {
    type Iter<'a>
        = PolygonIter<'a, T>
    where
        T: 'a;
    type ExteriorIter<'a>
        = LineStringIter<'a, T>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.exterior()
            .coords_iter()
            .chain(MapCoordsIter(self.interiors().iter(), marker::PhantomData).flatten())
    }

    /// Return the number of coordinates in the `Polygon`.
    fn coords_count(&self) -> usize {
        self.exterior().coords_count()
            + self
                .interiors()
                .iter()
                .map(|i| i.coords_count())
                .sum::<usize>()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.exterior().coords_iter()
    }
}

/// State for the PolygonIter
enum PolygonIterState {
    Exterior,
    Interior(usize), // Holds the current interior ring index
    Done,
}

/// Helper iterator for Polygon coordinates (exterior + interiors)
pub struct PolygonCoordIter<'a, P, BP>
where
    P: PolygonTraitExt<T: CoordNum>,
    BP: Borrow<P>,
{
    polygon: BP,
    state: PolygonIterState,
    ring_idx: usize,
    /// Current coordinate index within the current ring
    coord_index: usize,
    ring_size: usize,
    marker: marker::PhantomData<&'a P>,
}

impl<P, BP> PolygonCoordIter<'_, P, BP>
where
    P: PolygonTraitExt<T: CoordNum>,
    BP: Borrow<P>,
{
    fn new(polygon: BP) -> Self {
        let ring_size;
        let ring_idx;
        let initial_state = if let Some(exterior) = polygon.borrow().exterior_ext() {
            ring_size = exterior.num_coords();
            ring_idx = 0;
            PolygonIterState::Exterior
        } else if let Some(interior) = polygon.borrow().interior_ext(0) {
            ring_size = interior.num_coords();
            ring_idx = 1;
            PolygonIterState::Interior(0)
        } else {
            ring_size = 0;
            ring_idx = 0;
            PolygonIterState::Done
        };

        Self {
            polygon,
            state: initial_state,
            ring_idx,
            coord_index: 0,
            ring_size,
            marker: marker::PhantomData,
        }
    }

    fn start_interior_ring(&mut self, ring_idx: usize, num_coords: usize) {
        self.state = PolygonIterState::Interior(ring_idx);
        self.ring_idx = ring_idx;
        self.coord_index = 0;
        self.ring_size = num_coords;
    }
}

impl<P, BP> Iterator for PolygonCoordIter<'_, P, BP>
where
    P: PolygonTraitExt<T: CoordNum>,
    BP: Borrow<P>,
{
    type Item = Coord<P::T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (ring_idx, ring_size) = {
            let ring_opt = if self.ring_idx == 0 {
                self.polygon.borrow().exterior_ext()
            } else {
                self.polygon.borrow().interior_ext(self.ring_idx - 1)
            };
            if let Some(ring) = ring_opt {
                if self.coord_index < self.ring_size {
                    let coord = unsafe { ring.geo_coord_unchecked(self.coord_index) };
                    self.coord_index += 1;
                    return Some(coord);
                } else {
                    // Finished this ring, move to next
                    match self.state {
                        PolygonIterState::Exterior => {
                            let interior_opt = self.polygon.borrow().interior_ext(0);
                            match interior_opt {
                                Some(interior) => (1, interior.num_coords()),
                                None => return None,
                            }
                        }
                        PolygonIterState::Interior(ring_idx) => {
                            let interior_opt = self.polygon.borrow().interior_ext(ring_idx + 1);
                            match interior_opt {
                                Some(interior) => (ring_idx + 2, interior.num_coords()),
                                None => return None,
                            }
                        }
                        PolygonIterState::Done => return None,
                    }
                }
            } else {
                // No more rings
                return None;
            }
        };

        self.start_interior_ring(ring_idx, ring_size);
        self.next()
    }
}

impl<T, P> CoordsIterTrait<PolygonTag> for P
where
    T: CoordNum,
    P: PolygonTraitExt<T = T>,
{
    type Iter<'a>
        = PolygonCoordIter<'a, P, &'a P>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = LineStringCoordIter<P::RingTypeExt<'a>, P::RingTypeExt<'a>>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        PolygonCoordIter::new(self)
    }

    // Return the number of coordinates in the `Polygon`.
    fn coords_count_trait(&self) -> usize {
        self.exterior_ext()
            .map_or(0, |exterior| exterior.num_coords())
            + self.interiors_ext().map(|i| i.num_coords()).sum::<usize>()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        let exterior_opt = self.exterior_ext();
        LineStringCoordIter::new(exterior_opt)
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiPoint<T> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Point<T>>, Point<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPoint`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

pub struct MultiPointCoordIter<'a, MP>
where
    MP: MultiPointTraitExt<T: CoordNum>,
{
    mp: &'a MP,
    idx: usize,
    limit: usize,
}

impl<'a, MP> MultiPointCoordIter<'a, MP>
where
    MP: MultiPointTraitExt<T: CoordNum>,
{
    fn new(mp: &'a MP) -> Self {
        let limit = mp.num_points();
        Self { mp, idx: 0, limit }
    }
}

impl<MP> Iterator for MultiPointCoordIter<'_, MP>
where
    MP: MultiPointTraitExt<T: CoordNum>,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx >= self.limit {
                return None;
            }
            let coord = unsafe { self.mp.geo_coord_unchecked(self.idx) };
            self.idx += 1;
            if coord.is_some() {
                return coord;
            }
        }
    }
}

impl<T, MP> CoordsIterTrait<MultiPointTag> for MP
where
    T: CoordNum,
    MP: MultiPointTraitExt<T = T>,
{
    type Iter<'a>
        = MultiPointCoordIter<'a, MP>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        MultiPointCoordIter::new(self)
    }

    fn coords_count_trait(&self) -> usize {
        self.points_ext()
            .filter_map(|p| p.coord_ext().map(|_c| 1))
            .count()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter_trait()
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiLineString<T> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiLineString`.
    fn coords_count(&self) -> usize {
        self.0
            .iter()
            .map(|line_string| line_string.coords_count())
            .sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

pub struct MultiLineStringCoordIter<'a, MLS>
where
    MLS: MultiLineStringTraitExt<T: CoordNum>,
{
    ml: &'a MLS,
    idx_ls: usize,
    ls_opt: Option<MLS::LineStringTypeExt<'a>>,
    idx: usize,
    limit: usize,
}

impl<'a, T, MLS> MultiLineStringCoordIter<'a, MLS>
where
    T: CoordNum,
    MLS: MultiLineStringTraitExt<T = T>,
{
    fn new(ml: &'a MLS) -> Self {
        match ml.line_string_ext(0) {
            Some(ls) => {
                let limit = ls.num_coords();
                Self {
                    ml,
                    idx_ls: 0,
                    ls_opt: Some(ls),
                    idx: 0,
                    limit,
                }
            }
            None => Self {
                ml,
                idx_ls: 0,
                ls_opt: None,
                idx: 0,
                limit: 0,
            },
        }
    }
}

impl<T, MLS> Iterator for MultiLineStringCoordIter<'_, MLS>
where
    T: CoordNum,
    MLS: MultiLineStringTraitExt<T = T>,
{
    type Item = Coord<MLS::T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx < self.limit {
                // When idx < limit, ls_opt is guaranteed to exist. limit is the number of coordinates
                // in ls_opt and we have idx < limit, so the geo_coord_unchecked is guaranteed to be safe.
                let coord = unsafe { self.ls_opt.as_ref().unwrap().geo_coord_unchecked(self.idx) };
                self.idx += 1;
                return Some(coord);
            } else {
                // Head to the next line string
                let ls_opt = self.ml.line_string_ext(self.idx_ls + 1);
                match &ls_opt {
                    Some(ls) => {
                        self.idx = 0;
                        self.limit = ls.num_coords();
                        self.ls_opt = ls_opt;
                    }
                    None => return None,
                }
            }
        }
    }
}

impl<T, MLS> CoordsIterTrait<MultiLineStringTag> for MLS
where
    T: CoordNum,
    MLS: MultiLineStringTraitExt<T = T>,
{
    type Iter<'a>
        = MultiLineStringCoordIter<'a, MLS>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        MultiLineStringCoordIter::new(self)
    }

    fn coords_count_trait(&self) -> usize {
        self.line_strings_ext().map(|ls| ls.num_coords()).sum()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter_trait()
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiPolygon<T> {
    type Iter<'a>
        = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = iter::Flatten<MapExteriorCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPolygon`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|polygon| polygon.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        MapExteriorCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

pub struct MultiPolygonCoordIter<'a, MP>
where
    MP: MultiPolygonTraitExt<T: CoordNum>,
{
    mp: &'a MP,
    idx_poly: usize,
    poly_iter: Option<PolygonCoordIter<'a, MP::PolygonTypeExt<'a>, MP::PolygonTypeExt<'a>>>,
}

impl<'a, T, MP> MultiPolygonCoordIter<'a, MP>
where
    T: CoordNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    fn new(mp: &'a MP) -> Self {
        match mp.polygon_ext(0) {
            Some(poly) => Self {
                mp,
                idx_poly: 0,
                poly_iter: Some(PolygonCoordIter::new(poly)),
            },
            None => Self {
                mp,
                idx_poly: 0,
                poly_iter: None,
            },
        }
    }
}

impl<T, MP> Iterator for MultiPolygonCoordIter<'_, MP>
where
    T: CoordNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.poly_iter.as_mut() {
            Some(iter) => {
                let coord = iter.next();
                if coord.is_some() {
                    coord
                } else {
                    self.idx_poly += 1;
                    match self.mp.polygon_ext(self.idx_poly) {
                        Some(poly) => {
                            self.poly_iter = Some(PolygonCoordIter::new(poly));
                            self.next()
                        }
                        None => None,
                    }
                }
            }
            None => None,
        }
    }
}

pub struct MultiPolygonExteriorCoordIter<'a, MP>
where
    MP: MultiPolygonTraitExt<T: CoordNum>,
{
    mp: &'a MP,
    current_poly: Option<MP::PolygonTypeExt<'a>>,
    idx_poly: usize,
    idx: usize,
    limit: usize,
}

impl<'a, T, MP> MultiPolygonExteriorCoordIter<'a, MP>
where
    T: CoordNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    fn new(mp: &'a MP) -> Self {
        match mp.polygon_ext(0) {
            Some(poly) => {
                // limit will be zero if the exterior ring doesn't exist.
                let limit = poly.exterior_ext().map_or(0, |ring| ring.num_coords());
                Self {
                    mp,
                    idx_poly: 0,
                    idx: 0,
                    limit,
                    current_poly: Some(poly),
                }
            }
            None => Self {
                mp,
                idx_poly: 0,
                idx: 0,
                limit: 0,
                current_poly: None,
            },
        }
    }
}

impl<T, MP> Iterator for MultiPolygonExteriorCoordIter<'_, MP>
where
    T: CoordNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    type Item = Coord<MP::T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.limit {
            let coord = unsafe {
                // When idx < limit, current_poly and the exterior ring are guaranteed to exist.
                // This is because if either of them doesn't exist, limit would be 0, and we won't
                // reach here in this case.
                self.current_poly
                    .as_ref()
                    .unwrap()
                    .exterior_ext()
                    .unwrap()
                    .geo_coord_unchecked(self.idx)
            };
            self.idx += 1;
            Some(coord)
        } else {
            self.idx_poly += 1;
            match self.mp.polygon_ext(self.idx_poly) {
                Some(poly) => {
                    self.idx = 0;
                    // limit will be zero if the exterior ring doesn't exist.
                    self.limit = poly.exterior_ext().map_or(0, |ring| ring.num_coords());
                    self.current_poly = Some(poly);
                    self.next()
                }
                None => None,
            }
        }
    }
}

impl<T, MP> CoordsIterTrait<MultiPolygonTag> for MP
where
    T: CoordNum,
    MP: MultiPolygonTraitExt<T = T>,
{
    type Iter<'a>
        = MultiPolygonCoordIter<'a, MP>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = MultiPolygonExteriorCoordIter<'a, MP>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        MultiPolygonCoordIter::new(self)
    }

    fn coords_count_trait(&self) -> usize {
        // self.0.iter().map(|polygon| polygon.coords_count()).sum()
        self.polygons_ext().map(|p| p.coords_count_trait()).sum()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        MultiPolygonExteriorCoordIter::new(self)
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<T: CoordNum> CoordsIter for GeometryCollection<T> {
    type Iter<'a>
        = Box<dyn Iterator<Item = Coord<T>> + 'a>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Box<dyn Iterator<Item = Coord<T>> + 'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        Box::new(self.0.iter().flat_map(|geometry| geometry.coords_iter()))
    }

    /// Return the number of coordinates in the `GeometryCollection`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|geometry| geometry.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        Box::new(
            self.0
                .iter()
                .flat_map(|geometry| geometry.exterior_coords_iter()),
        )
    }
}

impl<T, GC> CoordsIterTrait<GeometryCollectionTag> for GC
where
    T: CoordNum,
    GC: GeometryCollectionTraitExt<T = T>,
{
    type Iter<'a>
        = std::vec::IntoIter<Coord<T>>
    where
        Self: 'a;

    type ExteriorIter<'a>
        = std::vec::IntoIter<Coord<T>>
    where
        Self: 'a;

    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        // Boxing is likely necessary here due to heterogeneous nature
        // and complexity of tracking state across different geometry types
        // without significant code complexity or allocations anyway.
        let mut all_coords: Vec<Coord<Self::Scalar>> = Vec::new();
        for g in self.geometries_ext() {
            all_coords.extend(g.coords_iter_trait());
        }
        all_coords.into_iter()
    }

    /// Return the number of coordinates in the `GeometryCollection`.
    fn coords_count_trait(&self) -> usize {
        self.geometries_ext().map(|g| g.coords_count_trait()).sum()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        let mut all_coords: Vec<Coord<Self::Scalar>> = Vec::new();
        for g in self.geometries_ext() {
            all_coords.extend(g.exterior_coords_iter_trait());
        }
        all_coords.into_iter()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

type RectIter<T> =
    iter::Chain<iter::Chain<CoordinateChainOnce<T>, iter::Once<Coord<T>>>, iter::Once<Coord<T>>>;

impl<T: CoordNum> CoordsIter for Rect<T> {
    type Iter<'a>
        = RectIter<T>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    /// Iterates over the coordinates in CCW order
    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(coord! {
            x: self.max().x,
            y: self.min().y,
        })
        .chain(iter::once(coord! {
            x: self.max().x,
            y: self.max().y,
        }))
        .chain(iter::once(coord! {
            x: self.min().x,
            y: self.max().y,
        }))
        .chain(iter::once(coord! {
            x: self.min().x,
            y: self.min().y,
        }))
    }

    /// Return the number of coordinates in the `Rect`.
    ///
    /// Note: Although a `Rect` is represented by two coordinates, it is
    /// spatially represented by four, so this method returns `4`.
    fn coords_count(&self) -> usize {
        4
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<T: CoordNum> CoordsIter for Triangle<T> {
    type Iter<'a>
        = iter::Chain<CoordinateChainOnce<T>, iter::Once<Coord<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
            .chain(iter::once(self.1))
            .chain(iter::once(self.2))
    }

    /// Return the number of coordinates in the `Triangle`.
    fn coords_count(&self) -> usize {
        3
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<T: CoordNum> CoordsIter for Geometry<T> {
    type Iter<'a>
        = GeometryCoordsIter<'a, T>
    where
        T: 'a;
    type ExteriorIter<'a>
        = GeometryExteriorCoordsIter<'a, T>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        match self {
            Geometry::Point(g) => GeometryCoordsIter::Point(g.coords_iter()),
            Geometry::Line(g) => GeometryCoordsIter::Line(g.coords_iter()),
            Geometry::LineString(g) => GeometryCoordsIter::LineString(g.coords_iter()),
            Geometry::Polygon(g) => GeometryCoordsIter::Polygon(g.coords_iter()),
            Geometry::MultiPoint(g) => GeometryCoordsIter::MultiPoint(g.coords_iter()),
            Geometry::MultiLineString(g) => GeometryCoordsIter::MultiLineString(g.coords_iter()),
            Geometry::MultiPolygon(g) => GeometryCoordsIter::MultiPolygon(g.coords_iter()),
            Geometry::GeometryCollection(g) => {
                GeometryCoordsIter::GeometryCollection(g.coords_iter())
            }
            Geometry::Rect(g) => GeometryCoordsIter::Rect(g.coords_iter()),
            Geometry::Triangle(g) => GeometryCoordsIter::Triangle(g.coords_iter()),
        }
    }
    crate::geometry_delegate_impl! {
        /// Return the number of coordinates in the `Geometry`.
        fn coords_count(&self) -> usize;
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        match self {
            Geometry::Point(g) => GeometryExteriorCoordsIter::Point(g.exterior_coords_iter()),
            Geometry::Line(g) => GeometryExteriorCoordsIter::Line(g.exterior_coords_iter()),
            Geometry::LineString(g) => {
                GeometryExteriorCoordsIter::LineString(g.exterior_coords_iter())
            }
            Geometry::Polygon(g) => GeometryExteriorCoordsIter::Polygon(g.exterior_coords_iter()),
            Geometry::MultiPoint(g) => {
                GeometryExteriorCoordsIter::MultiPoint(g.exterior_coords_iter())
            }
            Geometry::MultiLineString(g) => {
                GeometryExteriorCoordsIter::MultiLineString(g.exterior_coords_iter())
            }
            Geometry::MultiPolygon(g) => {
                GeometryExteriorCoordsIter::MultiPolygon(g.exterior_coords_iter())
            }
            Geometry::GeometryCollection(g) => {
                GeometryExteriorCoordsIter::GeometryCollection(g.exterior_coords_iter())
            }
            Geometry::Rect(g) => GeometryExteriorCoordsIter::Rect(g.exterior_coords_iter()),
            Geometry::Triangle(g) => GeometryExteriorCoordsIter::Triangle(g.exterior_coords_iter()),
        }
    }
}

impl<T, G> CoordsIterTrait<GeometryTag> for G
where
    T: CoordNum,
    G: GeometryTraitExt<T = T>,
{
    type Iter<'a>
        = GeometryTraitCoordsIter<'a, G>
    where
        Self: 'a;
    type ExteriorIter<'a>
        = GeometryTraitExteriorCoordsIter<'a, G>
    where
        Self: 'a;
    type Scalar = T;

    fn coords_iter_trait(&self) -> Self::Iter<'_> {
        match self.as_type_ext() {
            GeometryTypeExt::Point(g) => GeometryTraitCoordsIter::Point(g.coords_iter_trait()),
            GeometryTypeExt::Line(g) => GeometryTraitCoordsIter::Line(g.coords_iter_trait()),
            GeometryTypeExt::LineString(g) => {
                GeometryTraitCoordsIter::LineString(g.coords_iter_trait())
            }
            GeometryTypeExt::Polygon(g) => GeometryTraitCoordsIter::Polygon(g.coords_iter_trait()),
            GeometryTypeExt::MultiPoint(g) => {
                GeometryTraitCoordsIter::MultiPoint(g.coords_iter_trait())
            }
            GeometryTypeExt::MultiLineString(g) => {
                GeometryTraitCoordsIter::MultiLineString(g.coords_iter_trait())
            }
            GeometryTypeExt::MultiPolygon(g) => {
                GeometryTraitCoordsIter::MultiPolygon(g.coords_iter_trait())
            }
            GeometryTypeExt::GeometryCollection(g) => {
                GeometryTraitCoordsIter::GeometryCollection(g.coords_iter_trait())
            }
            _ => todo!(),
            // GeometryTypeExt::Rect(g) => GeometryCoordsIter::Rect(g.coords_iter()),
            // GeometryTypeExt::Triangle(g) => GeometryCoordsIter::Triangle(g.coords_iter()),
        }
    }
    // crate::geometry_trait_ext_delegate_impl! {
    //     /// Return the number of coordinates in the `Geometry`.
    //     fn coords_count_trait(&self) -> usize;
    // }

    fn coords_count_trait(&self) -> usize {
        todo!()
    }

    fn exterior_coords_iter_trait(&self) -> Self::ExteriorIter<'_> {
        match self.as_type_ext() {
            GeometryTypeExt::Point(g) => {
                GeometryTraitExteriorCoordsIter::Point(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::Line(g) => {
                GeometryTraitExteriorCoordsIter::Line(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::LineString(g) => {
                GeometryTraitExteriorCoordsIter::LineString(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::Polygon(g) => {
                GeometryTraitExteriorCoordsIter::Polygon(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::MultiPoint(g) => {
                GeometryTraitExteriorCoordsIter::MultiPoint(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::MultiLineString(g) => {
                GeometryTraitExteriorCoordsIter::MultiLineString(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::MultiPolygon(g) => {
                GeometryTraitExteriorCoordsIter::MultiPolygon(g.exterior_coords_iter_trait())
            }
            GeometryTypeExt::GeometryCollection(g) => {
                GeometryTraitExteriorCoordsIter::GeometryCollection(g.exterior_coords_iter_trait())
            }
            // GeometryTypeExt::Rect(g) => GeometryExteriorCoordsIter::Rect(g.exterior_coords_iter_trait()),
            // GeometryTypeExt::Triangle(g) => GeometryExteriorCoordsIter::Triangle(g.exterior_coords_iter_trait()),
            _ => todo!(),
        }
    }
}

// ┌──────────────────────────┐
// │ Implementation for Array │
// └──────────────────────────┘

impl<const N: usize, T: CoordNum> CoordsIter for [Coord<T>; N] {
    type Iter<'a>
        = iter::Copied<slice::Iter<'a, Coord<T>>>
    where
        T: 'a;
    type ExteriorIter<'a>
        = Self::Iter<'a>
    where
        T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    fn coords_count(&self) -> usize {
        N
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌──────────────────────────┐
// │ Implementation for Slice │
// └──────────────────────────┘

impl<'a, T: CoordNum> CoordsIter for &'a [Coord<T>] {
    type Iter<'b>
        = iter::Copied<slice::Iter<'b, Coord<T>>>
    where
        T: 'b,
        'a: 'b;
    type ExteriorIter<'b>
        = Self::Iter<'b>
    where
        T: 'b,
        'a: 'b;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    fn coords_count(&self) -> usize {
        self.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
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

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapExteriorCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
    for MapExteriorCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::ExteriorIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.exterior_coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryCoordsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as CoordsIter>::Iter<'a>),
    Line(<Line<T> as CoordsIter>::Iter<'a>),
    LineString(<LineString<T> as CoordsIter>::Iter<'a>),
    Polygon(<Polygon<T> as CoordsIter>::Iter<'a>),
    MultiPoint(<MultiPoint<T> as CoordsIter>::Iter<'a>),
    MultiLineString(<MultiLineString<T> as CoordsIter>::Iter<'a>),
    MultiPolygon(<MultiPolygon<T> as CoordsIter>::Iter<'a>),
    GeometryCollection(<GeometryCollection<T> as CoordsIter>::Iter<'a>),
    Rect(<Rect<T> as CoordsIter>::Iter<'a>),
    Triangle(<Triangle<T> as CoordsIter>::Iter<'a>),
}

impl<T: CoordNum> Iterator for GeometryCoordsIter<'_, T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryCoordsIter::Point(g) => g.next(),
            GeometryCoordsIter::Line(g) => g.next(),
            GeometryCoordsIter::LineString(g) => g.next(),
            GeometryCoordsIter::Polygon(g) => g.next(),
            GeometryCoordsIter::MultiPoint(g) => g.next(),
            GeometryCoordsIter::MultiLineString(g) => g.next(),
            GeometryCoordsIter::MultiPolygon(g) => g.next(),
            GeometryCoordsIter::GeometryCollection(g) => g.next(),
            GeometryCoordsIter::Rect(g) => g.next(),
            GeometryCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryCoordsIter::Point(g) => g.size_hint(),
            GeometryCoordsIter::Line(g) => g.size_hint(),
            GeometryCoordsIter::LineString(g) => g.size_hint(),
            GeometryCoordsIter::Polygon(g) => g.size_hint(),
            GeometryCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryCoordsIter::Rect(g) => g.size_hint(),
            GeometryCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

#[doc(hidden)]
pub enum GeometryTraitCoordsIter<'a, G>
where
    G: GeometryTraitExt<T: CoordNum> + 'a,
{
    Point(<G::PointTypeExt<'a> as CoordsIterTrait<PointTag>>::Iter<'a>),
    Line(<G::LineTypeExt<'a> as CoordsIterTrait<LineTag>>::Iter<'a>),
    LineString(<G::LineStringTypeExt<'a> as CoordsIterTrait<LineStringTag>>::Iter<'a>),
    Polygon(<G::PolygonTypeExt<'a> as CoordsIterTrait<PolygonTag>>::Iter<'a>),
    MultiPoint(<G::MultiPointTypeExt<'a> as CoordsIterTrait<MultiPointTag>>::Iter<'a>),
    MultiLineString(
        <G::MultiLineStringTypeExt<'a> as CoordsIterTrait<MultiLineStringTag>>::Iter<'a>,
    ),
    MultiPolygon(<G::MultiPolygonTypeExt<'a> as CoordsIterTrait<MultiPolygonTag>>::Iter<'a>),
    GeometryCollection(
        <G::GeometryCollectionTypeExt<'a> as CoordsIterTrait<GeometryCollectionTag>>::Iter<'a>,
    ),
    // Rect(<G::RectTypeExt<'a> as CoordsIterTrait<RectTag>>::Iter<'a>),
    // Triangle(<G::TriangleTypeExt<'a> as CoordsIterTrait<TriangleTag>>::Iter<'a>),
}

impl<'a, G> Iterator for GeometryTraitCoordsIter<'a, G>
where
    G: GeometryTraitExt<T: CoordNum> + 'a,
{
    type Item = Coord<G::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryTraitCoordsIter::Point(g) => g.next(),
            GeometryTraitCoordsIter::Line(g) => g.next(),
            GeometryTraitCoordsIter::LineString(g) => g.next(),
            GeometryTraitCoordsIter::Polygon(g) => g.next(),
            GeometryTraitCoordsIter::MultiPoint(g) => g.next(),
            GeometryTraitCoordsIter::MultiLineString(g) => g.next(),
            GeometryTraitCoordsIter::MultiPolygon(g) => g.next(),
            GeometryTraitCoordsIter::GeometryCollection(g) => g.next(),
            // GeometryTraitCoordsIter::Rect(g) => g.next(),
            // GeometryTraitCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryTraitCoordsIter::Point(g) => g.size_hint(),
            GeometryTraitCoordsIter::Line(g) => g.size_hint(),
            GeometryTraitCoordsIter::LineString(g) => g.size_hint(),
            GeometryTraitCoordsIter::Polygon(g) => g.size_hint(),
            GeometryTraitCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryTraitCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryTraitCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryTraitCoordsIter::GeometryCollection(g) => g.size_hint(),
            // GeometryTraitCoordsIter::Rect(g) => g.size_hint(),
            // GeometryTraitCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryExteriorCoordsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as CoordsIter>::ExteriorIter<'a>),
    Line(<Line<T> as CoordsIter>::ExteriorIter<'a>),
    LineString(<LineString<T> as CoordsIter>::ExteriorIter<'a>),
    Polygon(<Polygon<T> as CoordsIter>::ExteriorIter<'a>),
    MultiPoint(<MultiPoint<T> as CoordsIter>::ExteriorIter<'a>),
    MultiLineString(<MultiLineString<T> as CoordsIter>::ExteriorIter<'a>),
    MultiPolygon(<MultiPolygon<T> as CoordsIter>::ExteriorIter<'a>),
    GeometryCollection(<GeometryCollection<T> as CoordsIter>::ExteriorIter<'a>),
    Rect(<Rect<T> as CoordsIter>::ExteriorIter<'a>),
    Triangle(<Triangle<T> as CoordsIter>::ExteriorIter<'a>),
}

impl<T: CoordNum> Iterator for GeometryExteriorCoordsIter<'_, T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.next(),
            GeometryExteriorCoordsIter::Line(g) => g.next(),
            GeometryExteriorCoordsIter::LineString(g) => g.next(),
            GeometryExteriorCoordsIter::Polygon(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.next(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.next(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.next(),
            GeometryExteriorCoordsIter::Rect(g) => g.next(),
            GeometryExteriorCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Line(g) => g.size_hint(),
            GeometryExteriorCoordsIter::LineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Polygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Rect(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

#[doc(hidden)]
pub enum GeometryTraitExteriorCoordsIter<'a, G>
where
    G: GeometryTraitExt<T: CoordNum> + 'a,
{
    Point(<G::PointTypeExt<'a> as CoordsIterTrait<PointTag>>::ExteriorIter<'a>),
    Line(<G::LineTypeExt<'a> as CoordsIterTrait<LineTag>>::ExteriorIter<'a>),
    LineString(<G::LineStringTypeExt<'a> as CoordsIterTrait<LineStringTag>>::ExteriorIter<'a>),
    Polygon(<G::PolygonTypeExt<'a> as CoordsIterTrait<PolygonTag>>::ExteriorIter<'a>),
    MultiPoint(<G::MultiPointTypeExt<'a> as CoordsIterTrait<MultiPointTag>>::ExteriorIter<'a>),
    MultiLineString(
        <G::MultiLineStringTypeExt<'a> as CoordsIterTrait<MultiLineStringTag>>::ExteriorIter<'a>,
    ),
    MultiPolygon(
        <G::MultiPolygonTypeExt<'a> as CoordsIterTrait<MultiPolygonTag>>::ExteriorIter<'a>,
    ),
    GeometryCollection(
        <G::GeometryCollectionTypeExt<'a> as CoordsIterTrait<GeometryCollectionTag>>::ExteriorIter<
            'a,
        >,
    ),
    // Rect(<G::RectTypeExt<'a> as CoordsIterTrait<RectTag>>::ExteriorIter<'a>),
    // Triangle(<G::TriangleTypeExt<'a> as CoordsIterTrait<TriangleTag>>::ExteriorIter<'a>),
}

impl<'a, G> Iterator for GeometryTraitExteriorCoordsIter<'a, G>
where
    G: GeometryTraitExt<T: CoordNum> + 'a,
{
    type Item = Coord<G::T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryTraitExteriorCoordsIter::Point(g) => g.next(),
            GeometryTraitExteriorCoordsIter::Line(g) => g.next(),
            GeometryTraitExteriorCoordsIter::LineString(g) => g.next(),
            GeometryTraitExteriorCoordsIter::Polygon(g) => g.next(),
            GeometryTraitExteriorCoordsIter::MultiPoint(g) => g.next(),
            GeometryTraitExteriorCoordsIter::MultiLineString(g) => g.next(),
            GeometryTraitExteriorCoordsIter::MultiPolygon(g) => g.next(),
            GeometryTraitExteriorCoordsIter::GeometryCollection(g) => g.next(),
            // GeometryTraitExteriorCoordsIter::Rect(g) => g.next(),
            // GeometryTraitExteriorCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryTraitExteriorCoordsIter::Point(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::Line(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::LineString(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::Polygon(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryTraitExteriorCoordsIter::GeometryCollection(g) => g.size_hint(),
            // GeometryTraitExteriorCoordsIter::Rect(g) => g.size_hint(),
            // GeometryTraitExteriorCoordsIter::Triangle(g) => g.size_hint(),
        }
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
    }

    #[test]
    fn test_line() {
        let line = Line::new(coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. });

        let coords = line.coords_iter().collect::<Vec<_>>();

        assert_eq!(
            vec![coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. },],
            coords
        );
    }

    #[test]
    fn test_line_string() {
        let (line_string, expected_coords) = create_line_string();

        let actual_coords = line_string.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_polygon() {
        let (polygon, expected_coords) = create_polygon();

        let actual_coords = polygon.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_point() {
        let mut expected_coords = vec![];
        let (point, mut coords) = create_point();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiPoint::new(vec![point, point])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_line_string() {
        let mut expected_coords = vec![];
        let (line_string, mut coords) = create_line_string();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiLineString::new(vec![line_string.clone(), line_string])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_polygon() {
        let mut expected_coords = vec![];
        let (polygon, mut coords) = create_polygon();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiPolygon::new(vec![polygon.clone(), polygon])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_geometry() {
        let (line_string, expected_coords) = create_line_string();

        let actual_coords = Geometry::LineString(line_string)
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_rect() {
        let (rect, expected_coords) = create_rect();

        let actual_coords = rect.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_triangle() {
        let (triangle, expected_coords) = create_triangle();

        let actual_coords = triangle.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_geometry_collection() {
        let mut expected_coords = vec![];
        let (line_string, mut coords) = create_line_string();
        expected_coords.append(&mut coords);
        let (polygon, mut coords) = create_polygon();
        expected_coords.append(&mut coords);

        let actual_coords = GeometryCollection::new_from(vec![
            Geometry::LineString(line_string),
            Geometry::Polygon(polygon),
        ])
        .coords_iter()
        .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_array() {
        let coords = [
            coord! { x: 1., y: 2. },
            coord! { x: 3., y: 4. },
            coord! { x: 5., y: 6. },
        ];

        let actual_coords = coords.coords_iter().collect::<Vec<_>>();

        assert_eq!(coords.to_vec(), actual_coords);
    }

    #[test]
    fn test_slice() {
        let coords = &[
            coord! { x: 1., y: 2. },
            coord! { x: 3., y: 4. },
            coord! { x: 5., y: 6. },
        ];

        let actual_coords = coords.coords_iter().collect::<Vec<_>>();

        assert_eq!(coords.to_vec(), actual_coords);
    }

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
            vec![
                coord! { x: 1., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 5., y: 6. },
            ],
        )
    }

    fn create_rect() -> (Rect, Vec<Coord>) {
        (
            Rect::new(coord! { x: 1., y: 2. }, coord! { x: 3., y: 4. }),
            vec![
                coord! { x: 3., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 1., y: 4. },
                coord! { x: 1., y: 2. },
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
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 5.0, y: 10.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 1.0, y: 1.0 },
                coord! { x: 9.0, y: 1.0 },
                coord! { x: 5.0, y: 9.0 },
                coord! { x: 1.0, y: 1.0 },
            ],
        )
    }
}
