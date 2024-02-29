use geo_types::*;

use crate::{Closest, EuclideanDistance, GeoFloat};

/// Mostly the same as `ClosestPointPreciseInfo` with some more info on the closest point useful
/// for comparison of results but generally too unergonomic to deal with
///
/// (since we would need to get the point out of the `Closest` context a lot of times, causing
/// unwraps all over the place)
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClosestPointInfo<F: GeoFloat> {
    pub(crate) from_linestring: ConnectionKind,
    pub(crate) point_in_self: Point<F>,
    /// closest point to the `point_in_hole` point (with some extra information, `Closest` context)
    pub(crate) point_in_other: Closest<F>,
}

/// This struct holds all the data that is needed to describe the closest connection between a
/// point in the first hole of the polygon and another point either in the exterior or another hole
/// of the polygon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClosestPointPreciseInfo<F: GeoFloat> {
    /// this field classifies the target (exterior or other hole). If it's another hole it also
    /// provides closer information which index it has in the `.interiors()` return vector
    pub(crate) from_linestring: ConnectionKind,
    /// the point in the first hole of the polygon. This data is needed to draw the connecting line
    pub(crate) point_in_self: Point<F>,
    /// closest point to the `point_in_hole` point
    pub(crate) point_in_other: Point<F>,
}

impl<F: GeoFloat> ClosestPointPreciseInfo<F> {
    // conversion function, fails if closest value was Indeterminate
    pub(crate) fn from_unprecise(value: ClosestPointInfo<F>) -> Option<Self> {
        Some(ClosestPointPreciseInfo {
            from_linestring: value.from_linestring,
            point_in_self: value.point_in_self,
            point_in_other: unpack_closest(value)?,
        })
    }
}

pub(crate) struct PolyAndClosest<F: GeoFloat> {
    pub(crate) poly: Polygon<F>,
    pub(crate) closest: ClosestPointPreciseInfo<F>,
}

/// type that allows a retry with the same arguments in the case of an error
pub(crate) type Retry<F> = Result<MultiPolygon<F>, PolyAndClosest<F>>;

/// This struct holds all the data that is needed to describe the closest connection between a
/// point in the first hole of the polygon and another point either in the exterior or another hole
/// of the polygon
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClosestPointPreciseInfoOuterBanana {
    // indices of points of first poly
    pub(crate) half_1: Vec<usize>,
    // indices of points of second poly
    pub(crate) half_2: Vec<usize>,
}

/// Human readable enum to:
///
/// - distinguish between exterior and interior cases
/// - hold information which interior we're talking about in the interior case
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectionKind {
    Exterior,
    Interior(usize),
}

impl ConnectionKind {
    /// This function handles incoming indices as follows
    ///
    /// - index 0 => exterior
    /// - index 1..n => hole 1..n
    ///
    /// Note that this excludes the first hole (index 0)
    pub(crate) fn from_normal_index(n: usize) -> Self {
        if n == 0 {
            Self::Exterior
        } else {
            // in this case we enumerate all but the first
            Self::Interior(n)
        }
    }

    /// This function handles incoming indices as follows
    ///
    /// - index 0 => exterior
    /// - index 1..(n+1) => hole 0..n
    ///
    /// Note that this includes the first hole (index 0)
    pub(crate) fn from_banana_index(n: usize) -> Self {
        if n == 0 {
            Self::Exterior
        } else {
            Self::Interior(n.saturating_sub(1))
        }
    }
}

/// returns:
///
/// - None for `Indeterminate` cases
/// - `Some(input)` for valid cases
pub(crate) fn filter_out_indeterminate<F: GeoFloat>(
    c: ClosestPointInfo<F>,
) -> Option<ClosestPointInfo<F>> {
    match c.point_in_other {
        Closest::Intersection(_) => Some(c),
        Closest::SinglePoint(_) => Some(c),
        Closest::Indeterminate => None,
    }
}

/// returns:
///
/// - None for `Indeterminate` cases
/// - `Some(point)` for valid cases
pub(crate) fn unpack_closest<F: GeoFloat>(c: ClosestPointInfo<F>) -> Option<Point<F>> {
    match c.point_in_other {
        Closest::Intersection(p) => Some(p),
        Closest::SinglePoint(p) => Some(p),
        Closest::Indeterminate => None,
    }
}

// copied from the impl of Closest::best_of_two and slightly adjusted here
/// compares two closest points and returns the best of the two, i.e. the one which is closer to
/// it's target
pub(crate) fn best_of_two<F: GeoFloat>(
    a: ClosestPointInfo<F>,
    b: ClosestPointInfo<F>,
) -> ClosestPointInfo<F> {
    let inner_pa = match a.point_in_other {
        Closest::Indeterminate => return b,
        Closest::Intersection(_) => return a,
        Closest::SinglePoint(l) => l,
    };
    let inner_pb = match b.point_in_other {
        Closest::Indeterminate => return a,
        Closest::Intersection(_) => return b,
        Closest::SinglePoint(r) => r,
    };

    if inner_pa.euclidean_distance(&a.point_in_self)
        <= inner_pb.euclidean_distance(&b.point_in_self)
    {
        a
    } else {
        b
    }
}

/// helper function to reduce a list of closest points to the minimum
pub(crate) fn fold_closest<F: GeoFloat>(
    acc: Option<ClosestPointInfo<F>>,
    new: ClosestPointInfo<F>,
) -> Option<ClosestPointInfo<F>> {
    let new_best = acc.map_or(new, |old| best_of_two(old, new));
    filter_out_indeterminate(new_best)
}

/// helper function to reduce a list of closest points to the minimum
pub(crate) fn fold_closest_precise<F: GeoFloat>(
    acc: Option<ClosestPointPreciseInfo<F>>,
    new: ClosestPointPreciseInfo<F>,
) -> Option<ClosestPointPreciseInfo<F>> {
    acc.map_or(Some(new), |old| {
        let new_dist = new.point_in_other.euclidean_distance(&new.point_in_self);
        let old_dist = old.point_in_other.euclidean_distance(&old.point_in_self);
        let res = if new_dist < old_dist { new } else { old };
        Some(res)
    })
}
