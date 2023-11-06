use crate::algorithm::intersects::Intersects;
use crate::algorithm::stitch::Stitch;
pub use crate::spade_boolops::error::{SpadeBoolopsError, SpadeBoolopsResult};
use crate::spade_boolops::trait_def::SpadeBoolops;
use geo_types::{MultiPolygon, Polygon, Triangle};

use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::{OpType, TriangulateSpade};

impl<T> SpadeBoolops<T> for Polygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        _op_type: OpType,
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        vec![p1.clone(), p2.clone()]
            .constrained_outer_triangulation(Default::default())
            .map_err(SpadeBoolopsError::TriangulationError)?
            .into_iter()
            .filter(|tri| op_pred(tri))
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>()
            .stitch_together()
            .map_err(SpadeBoolopsError::StitchError)
    }
}

impl<T> SpadeBoolops<T> for MultiPolygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        op_type: OpType,
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        // helper function which helps to select only some polys of a multipolygon which pass a
        // certain predicate filter
        fn polys_with<T: SpadeTriangulationFloat, G>(
            mp1: &MultiPolygon<T>,
            filter: G,
        ) -> Vec<Polygon<T>>
        where
            G: Fn(&&Polygon<T>) -> bool,
        {
            // this clone here is needed since we can't construct the Multipolygons below otherwise
            // maybe if constrained_outer_triangulation would accept references, we could get rid
            // of this
            mp1.iter().filter(filter).cloned().collect::<Vec<_>>()
        }

        // gets only those polygons from mp1 which intersect any of the polygons in mp2
        let intersecting_polys = |mp1: &MultiPolygon<T>, mp2: &Vec<Polygon<T>>| {
            polys_with(mp1, move |p| mp2.iter().any(|o| p.intersects(o)))
        };

        let p1_inter = intersecting_polys(p1, &p2.0);
        // we know p2 can only intersect polys in p1_inter
        let p2_inter = intersecting_polys(p2, &p1_inter);

        // do the real boolean operation only on the intersecting parts
        let boolop_result = vec![MultiPolygon::new(p1_inter), MultiPolygon::new(p2_inter)]
            .constrained_outer_triangulation(Default::default())
            .map_err(SpadeBoolopsError::TriangulationError)?
            .into_iter()
            .filter(|tri| op_pred(tri))
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>();

        let non_intersecting_polys = |p1: &MultiPolygon<T>, p2: &MultiPolygon<T>| {
            polys_with(p1, move |p| !p2.iter().any(|o| p.intersects(o)))
        };

        // - if we union or difference, then we want to include non intersecting polys of the first
        //   argument multi polygon in the result
        // - if we union , then we want to include non intersecting polys of the second argument
        //   multi polygon in the result
        let p1_non_inter = matches!(op_type, OpType::Union | OpType::Difference)
            .then(|| non_intersecting_polys(p1, p2))
            .unwrap_or_default();
        let p2_non_inter = matches!(op_type, OpType::Union)
            .then(|| non_intersecting_polys(p2, p1))
            .unwrap_or_default();

        // do a constrained triangulation and then stitch the triangles together
        //
        // the triangulation is needed since the non intersecting polygons are not triangles with
        // the same characteristics as the boolop_result triangles and the stitch_together function
        // gets confused otherwise in some edge case tests
        [boolop_result, p1_non_inter, p2_non_inter]
            .concat()
            .constrained_triangulation(Default::default())
            .map_err(SpadeBoolopsError::TriangulationError)?
            .stitch_together()
            .map_err(SpadeBoolopsError::StitchError)
    }
}
