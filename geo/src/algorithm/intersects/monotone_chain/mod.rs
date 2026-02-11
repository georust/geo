//! Implements `Intersects` for MonotoneChain Geometry types
//!
//! Has optimized implementations when both sides are [`MonotoneChain`] backed
//! Falls back to base geometry when [`MonotoneChain`] is not supported on one side

// `Geom` intersects `MonotoneChain` and `MonotoneChainSegment` requires adding a lifetime to the macro
// duplicated from `intersects/mod.rs` but with additional lifetime generic
macro_rules! symmetric_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<'a, T> $crate::Intersects<$k> for $t
        where
            $k: $crate::Intersects<$t>,
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                rhs.intersects(self)
            }
        }
    };
}

// delegate the monotone chain backed geometry intersects to the base geometry
macro_rules! delegate_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<'a, T> $crate::Intersects<$k> for $t
        where
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                self.geometry().intersects(rhs)
            }
        }
    };
}

macro_rules! chains_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<'a, T> $crate::Intersects<$k> for $t
        where
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                use crate::MonotoneChains;
                rhs.chains()
                    .any(|c| self.chains().any(|c2| c.intersects(c2)))
            }
        }
    };
}

#[allow(clippy::module_inception)]
mod monotone_chain;
mod monotone_chain_geometry;
mod monotone_chain_linestring;
mod monotone_chain_multilinestring;
mod monotone_chain_multipolygon;
mod monotone_chain_polygon;
mod monotone_chain_segment;

#[cfg(test)]
mod tests {
    use crate::geometry::*;
    use crate::monotone_chain::*;
    use crate::wkt;
    use crate::{Convert, Intersects, Relate};

    #[test]
    fn test_intersects_edgecase() {
        let pt: Point<f64> = wkt! {POINT(0 0)}.convert();

        let ls0: LineString<f64> = LineString::empty();
        let ls1: LineString<f64> = wkt! {LINESTRING(0 0)}.convert();
        let ls2: LineString<f64> = wkt! {LINESTRING(0 0,1 1)}.convert();

        let mcs0: MonotoneChain<f64> = (&ls0).into();
        let mcs1: MonotoneChain<f64> = (&ls1).into();
        let mcs2: MonotoneChain<f64> = (&ls2).into();

        assert_eq!(ls0.intersects(&pt.0), mcs0.intersects(&pt.0));
        assert_eq!(ls1.intersects(&pt.0), ls1.relate(&pt).is_intersects());
        assert_eq!(ls1.intersects(&pt.0), mcs1.intersects(&pt.0));
        assert_eq!(ls2.intersects(&pt.0), mcs2.intersects(&pt.0));
    }

    #[test]
    fn test_exhaustive_compile_test() {
        // data
        let c: Coord<i32> = Coord { x: 0, y: 0 };
        let pt: Point<i32> = wkt!(POINT(0 0)).convert();
        let mpt: MultiPoint<i32> = wkt!(MULTIPOINT(0 0)).convert();

        let ls: LineString<i32> = wkt!(LINESTRING(0 0,1 1)).convert();
        let multi_ls: MultiLineString<i32> = wkt!(MULTILINESTRING((0 0,1 1))).convert();
        let ln: Line<i32> = wkt!(LINE(0 0,1 1)).convert();

        let poly: Polygon<i32> = wkt! { POLYGON((0 0,1 1,1 0,0 0)) }.convert();
        let multi_poly: MultiPolygon<i32> = wkt! { MULTIPOLYGON(((0 0,1 1,1 0,0 0))) }.convert();
        let rect: Rect<i32> = wkt! { RECT(10 20,30 10) }.convert();
        let tri: Triangle<i32> = wkt! { TRIANGLE(0 0,10 20,20 -10) }.convert();

        let geom: Geometry<i32> = Geometry::Point(pt);
        let gc: GeometryCollection<i32> = GeometryCollection::new_from(vec![geom.clone()]);

        let m_ls: MonotoneChainLineString<i32> = (&ls).into();
        let m_mls: MonotoneChainMultiLineString<i32> = (&multi_ls).into();
        let m_poly: MonotoneChainPolygon<i32> = (&poly).into();
        let m_mpoly: MonotoneChainMultiPolygon<i32> = (&multi_poly).into();

        let _ = m_ls.intersects(&m_ls);
        let _ = m_ls.intersects(&m_mls);
        let _ = m_ls.intersects(&m_poly);
        let _ = m_ls.intersects(&m_mpoly);

        let _ = m_mls.intersects(&m_ls);
        let _ = m_mls.intersects(&m_mls);
        let _ = m_mls.intersects(&m_poly);
        let _ = m_mls.intersects(&m_mpoly);

        let _ = m_poly.intersects(&m_ls);
        let _ = m_poly.intersects(&m_mls);
        let _ = m_poly.intersects(&m_poly);
        let _ = m_poly.intersects(&m_mpoly);

        let _ = m_mpoly.intersects(&m_ls);
        let _ = m_mpoly.intersects(&m_mls);
        let _ = m_mpoly.intersects(&m_poly);
        let _ = m_mpoly.intersects(&m_mpoly);
    }
}
