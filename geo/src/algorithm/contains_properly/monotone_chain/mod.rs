//! Implements `ContainsProperly` for [`MonotoneChain`] backed geometries
//! Falls back to `ContainsProperly` if no monotone chain based implementation is available

use crate::MonotoneChains;
use crate::{GeoNum, Intersects};

mod multipolygon;
mod polygon;

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

#[cfg(test)]
mod test {
    use crate::ContainsProperly;
    use crate::monotone_chain::geometry::*;
    use geo_types::*;

    #[test]
    fn exhaustive_compile_test() {
        // geo types
        let poly = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        // monotone types
        let m_poly: MonotoneChainPolygon<f64> = (&poly).into();
        let m_multi_poly: MonotoneChainMultiPolygon<f64> = (&multi_poly).into();

        // forward m_poly
        let _ = m_poly.contains_properly(&m_poly);
        let _ = m_multi_poly.contains_properly(&m_poly);

        // backward m_poly
        let _ = m_poly.contains_properly(&m_poly);
        let _ = m_poly.contains_properly(&m_multi_poly);

        // forward m_multi_poly
        let _ = m_poly.contains_properly(&m_multi_poly);
        let _ = m_multi_poly.contains_properly(&m_multi_poly);

        // backward m_multi_poly
        let _ = m_multi_poly.contains_properly(&m_poly);
        let _ = m_multi_poly.contains_properly(&m_multi_poly);
    }
}
