//! Implements `ContainsProperly` for [`MonotoneChain`] backed geometries
//! Falls back to `ContainsProperly` if no monotone chain based implementation is available

use crate::coordinate_position::{CoordPos, CoordinatePosition};
use crate::monotone_chain::coord_pos_relative_to_ring;
use crate::{
    ContainsProperly, Coord, GeoNum, HasDimensions, Intersects, LinesIter, MonotoneChainPolygon,
    MonotoneChains,
};

macro_rules! impl_contains_properly_target_monotone {
    ($target:ty,  [$($for:ty),*]) => {
        $(
            impl<'a, T> ContainsProperly<$target> for $for
            where
                T: GeoNum,
                $for:
                    ContainsProperly<LineString<T>> +
                    ContainsProperly<MultiLineString<T>> +
                    ContainsProperly<Polygon<T>> +
                    ContainsProperly<MultiPolygon<T>>
            {
                fn contains_properly(&self, rhs: &$target) -> bool {
                    self.contains_properly(rhs.geometry())
                }
            }
        )*
    };
}

macro_rules! impl_contains_properly_for_monotone {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<'a, T> ContainsProperly<$target> for $for
            where
                T: GeoNum,
                LineString<T> : ContainsProperly<$target>,
                MultiLineString<T> : ContainsProperly<$target>,
                Polygon<T> : ContainsProperly<$target>,
                MultiPolygon<T> : ContainsProperly<$target>
            {
                fn contains_properly(&self, rhs: &$target) -> bool {
                    self.geometry().contains_properly(rhs)
                }
            }
        )*
    };
}

mod geometry;
mod line_string;
mod multilinestring;
mod multipolygon;
mod polygon;

impl<'a, T: GeoNum> ContainsProperly<Coord<T>> for MonotoneChainPolygon<'a, T> {
    fn contains_properly(&self, rhs: &Coord<T>) -> bool {
        if self.is_empty() {
            return false;
        }
        self.coordinate_position(rhs) == CoordPos::Inside
    }
}

/// Return true if the boundary of lhs intersects any of the boundaries of rhs
/// where lhs and rhs are both polygons/multipolygons
/// This is a short circuit version of boundary_intersects which doesn't use the monotone algorithm
fn boundary_intersects<'a: 'caller, 'caller, T, G1, G2>(lhs: &'caller G1, rhs: &'caller G2) -> bool
where
    T: GeoNum + 'a,
    G1: MonotoneChains<'a, 'caller, T>,
    G2: MonotoneChains<'a, 'caller, T>,
{
    // use monotone when larger
    let rhs_arr = rhs.chains().collect::<Vec<_>>();

    lhs.chains()
        .any(|lhs| rhs_arr.iter().any(|rhs| lhs.intersects(*rhs)))
}

/// Given two non-empty polygons with no intersecting boundaries,
/// Return true if first polygon completely contains second polygon
pub(crate) fn polygon_polygon_inner_loop<T>(
    self_poly: &MonotoneChainPolygon<'_, T>,
    rhs_poly: &MonotoneChainPolygon<'_, T>,
) -> bool
where
    T: GeoNum,
{
    debug_assert!(!self_poly.is_empty() && !rhs_poly.is_empty());
    debug_assert!(
        self_poly
            .lines_iter()
            .flat_map(|s| rhs_poly.lines_iter().map(move |s2| (s, s2)))
            .all(|(s, s2)| !s.intersects(&s2))
    );

    let Some(rhs_ext_coord) = rhs_poly.exterior().geometry().0.first() else {
        return false;
    };

    if !self_poly.contains_properly(rhs_ext_coord) {
        // is disjoint
        return false;
    }

    // if there exits a self_hole which is not inside a rhs_hole
    // then there must be some point of rhs which does not lie on the interior of self
    // and hence self does not contains_properly rhs
    for self_hole in self_poly.interiors() {
        // empty hole is always covered
        let Some(self_hole_first_coord) = self_hole.geometry().0.first() else {
            continue;
        };

        // hole outside of RHS does not affect intersection
        if coord_pos_relative_to_ring(*self_hole_first_coord, rhs_poly.exterior())
            != CoordPos::Inside
        {
            continue;
        }

        // if any RHS point is inside self_hole, then we fail the contains_properly test
        // since all rings are either concentric or disjoint, we can check using representative point
        let self_hole_pt_in_rhs_hole = rhs_poly
            .interiors()
            .iter()
            .map(|rhs_ring| coord_pos_relative_to_ring(*self_hole_first_coord, rhs_ring))
            .any(|pos| pos == CoordPos::Inside);
        if !self_hole_pt_in_rhs_hole {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use crate::ContainsProperly;
    use crate::monotone_chain::geometry::*;
    use geo_types::*;

    #[test]
    fn exhaustive_compile_test() {
        // geo types
        let pt: Point = Point::new(0., 0.);
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let ln: Line = Line::new((0., 0.), (1., 1.));

        let poly = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
        let rect = Rect::new(coord! { x: 10., y: 20. }, coord! { x: 30., y: 10. });
        let tri = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 10., y: 20. },
            coord! { x: 20., y: -10. },
        );
        let geom = Geometry::Point(pt);
        let gc = GeometryCollection::new_from(vec![geom.clone()]);
        let multi_point = MultiPoint::new(vec![pt]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        // monotone types
        let m_ls: MonotoneChainLineString<f64> = (&ls).into();
        let m_multi_ls: MonotoneChainMultiLineString<f64> = (&multi_ls).into();
        let m_poly: MonotoneChainPolygon<f64> = (&poly).into();
        let m_multi_poly: MonotoneChainMultiPolygon<f64> = (&multi_poly).into();

        // forward m_ls
        let _ = pt.contains_properly(&m_ls);
        let _ = multi_point.contains_properly(&m_ls);
        let _ = ln.contains_properly(&m_ls);
        let _ = ls.contains_properly(&m_ls);
        let _ = multi_ls.contains_properly(&m_ls);
        let _ = poly.contains_properly(&m_ls);
        let _ = rect.contains_properly(&m_ls);
        let _ = tri.contains_properly(&m_ls);
        let _ = multi_poly.contains_properly(&m_ls);
        let _ = geom.contains_properly(&m_ls);
        let _ = gc.contains_properly(&m_ls);

        let _ = m_ls.contains_properly(&m_ls);
        let _ = m_multi_ls.contains_properly(&m_ls);
        let _ = m_poly.contains_properly(&m_ls);
        let _ = m_multi_poly.contains_properly(&m_ls);

        // backward m_ls
        let _ = m_ls.contains_properly(&pt);
        let _ = m_ls.contains_properly(&multi_point);
        let _ = m_ls.contains_properly(&ln);
        let _ = m_ls.contains_properly(&ls);
        let _ = m_ls.contains_properly(&multi_ls);
        let _ = m_ls.contains_properly(&poly);
        let _ = m_ls.contains_properly(&rect);
        let _ = m_ls.contains_properly(&tri);
        let _ = m_ls.contains_properly(&multi_poly);
        let _ = m_ls.contains_properly(&geom);
        let _ = m_ls.contains_properly(&gc);

        let _ = m_ls.contains_properly(&m_ls);
        let _ = m_ls.contains_properly(&m_multi_ls);
        let _ = m_ls.contains_properly(&m_poly);
        let _ = m_ls.contains_properly(&m_multi_poly);

        // forward m_multi_ls
        let _ = pt.contains_properly(&m_multi_ls);
        let _ = multi_point.contains_properly(&m_multi_ls);
        let _ = ln.contains_properly(&m_multi_ls);
        let _ = ls.contains_properly(&m_multi_ls);
        let _ = multi_ls.contains_properly(&m_multi_ls);
        let _ = poly.contains_properly(&m_multi_ls);
        let _ = rect.contains_properly(&m_multi_ls);
        let _ = tri.contains_properly(&m_multi_ls);
        let _ = multi_poly.contains_properly(&m_multi_ls);
        let _ = geom.contains_properly(&m_multi_ls);
        let _ = gc.contains_properly(&m_multi_ls);

        let _ = m_ls.contains_properly(&m_multi_ls);
        let _ = m_multi_ls.contains_properly(&m_multi_ls);
        let _ = m_poly.contains_properly(&m_multi_ls);
        let _ = m_multi_poly.contains_properly(&m_multi_ls);

        // backward m_multi_ls
        let _ = m_multi_ls.contains_properly(&pt);
        let _ = m_multi_ls.contains_properly(&multi_point);
        let _ = m_multi_ls.contains_properly(&ln);
        let _ = m_multi_ls.contains_properly(&ls);
        let _ = m_multi_ls.contains_properly(&multi_ls);
        let _ = m_multi_ls.contains_properly(&poly);
        let _ = m_multi_ls.contains_properly(&rect);
        let _ = m_multi_ls.contains_properly(&tri);
        let _ = m_multi_ls.contains_properly(&multi_poly);
        let _ = m_multi_ls.contains_properly(&geom);
        let _ = m_multi_ls.contains_properly(&gc);

        let _ = m_multi_ls.contains_properly(&m_ls);
        let _ = m_multi_ls.contains_properly(&m_multi_ls);
        let _ = m_multi_ls.contains_properly(&m_poly);
        let _ = m_multi_ls.contains_properly(&m_multi_poly);

        // forward m_poly
        let _ = pt.contains_properly(&m_poly);
        let _ = multi_point.contains_properly(&m_poly);
        let _ = ln.contains_properly(&m_poly);
        let _ = ls.contains_properly(&m_poly);
        let _ = multi_ls.contains_properly(&m_poly);
        let _ = poly.contains_properly(&m_poly);
        let _ = rect.contains_properly(&m_poly);
        let _ = tri.contains_properly(&m_poly);
        let _ = multi_poly.contains_properly(&m_poly);
        let _ = geom.contains_properly(&m_poly);
        let _ = gc.contains_properly(&m_poly);

        let _ = m_ls.contains_properly(&m_poly);
        let _ = m_multi_ls.contains_properly(&m_poly);
        let _ = m_poly.contains_properly(&m_poly);
        let _ = m_multi_poly.contains_properly(&m_poly);

        // backward m_poly
        let _ = m_poly.contains_properly(&pt);
        let _ = m_poly.contains_properly(&multi_point);
        let _ = m_poly.contains_properly(&ln);
        let _ = m_poly.contains_properly(&ls);
        let _ = m_poly.contains_properly(&multi_ls);
        let _ = m_poly.contains_properly(&poly);
        let _ = m_poly.contains_properly(&rect);
        let _ = m_poly.contains_properly(&tri);
        let _ = m_poly.contains_properly(&multi_poly);
        let _ = m_poly.contains_properly(&geom);
        let _ = m_poly.contains_properly(&gc);

        let _ = m_poly.contains_properly(&m_ls);
        let _ = m_poly.contains_properly(&m_multi_ls);
        let _ = m_poly.contains_properly(&m_poly);
        let _ = m_poly.contains_properly(&m_multi_poly);

        // forward m_multi_poly
        let _ = pt.contains_properly(&m_multi_poly);
        let _ = multi_point.contains_properly(&m_multi_poly);
        let _ = ln.contains_properly(&m_multi_poly);
        let _ = ls.contains_properly(&m_multi_poly);
        let _ = multi_ls.contains_properly(&m_multi_poly);
        let _ = poly.contains_properly(&m_multi_poly);
        let _ = rect.contains_properly(&m_multi_poly);
        let _ = tri.contains_properly(&m_multi_poly);
        let _ = multi_poly.contains_properly(&m_multi_poly);
        let _ = geom.contains_properly(&m_multi_poly);
        let _ = gc.contains_properly(&m_multi_poly);

        let _ = m_ls.contains_properly(&m_multi_poly);
        let _ = m_multi_ls.contains_properly(&m_multi_poly);
        let _ = m_poly.contains_properly(&m_multi_poly);
        let _ = m_multi_poly.contains_properly(&m_multi_poly);

        // backward m_multi_poly
        let _ = m_multi_poly.contains_properly(&pt);
        let _ = m_multi_poly.contains_properly(&multi_point);
        let _ = m_multi_poly.contains_properly(&ln);
        let _ = m_multi_poly.contains_properly(&ls);
        let _ = m_multi_poly.contains_properly(&multi_ls);
        let _ = m_multi_poly.contains_properly(&poly);
        let _ = m_multi_poly.contains_properly(&rect);
        let _ = m_multi_poly.contains_properly(&tri);
        let _ = m_multi_poly.contains_properly(&multi_poly);
        let _ = m_multi_poly.contains_properly(&geom);
        let _ = m_multi_poly.contains_properly(&gc);

        let _ = m_multi_poly.contains_properly(&m_ls);
        let _ = m_multi_poly.contains_properly(&m_multi_ls);
        let _ = m_multi_poly.contains_properly(&m_poly);
        let _ = m_multi_poly.contains_properly(&m_multi_poly);
    }
}
