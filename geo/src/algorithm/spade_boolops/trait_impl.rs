use crate::algorithm::stitch::Stitch;
pub use crate::spade_boolops::error::{SpadeBoolopsError, SpadeBoolopsResult};
use crate::spade_boolops::trait_def::SpadeBoolops;
use geo_types::{MultiPolygon, Polygon, Triangle};

use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::TriangulateSpade;

impl<T> SpadeBoolops<T> for Polygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        vec![p1.clone(), p2.clone()]
            .constrained_outer_triangulation()
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
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        vec![p1.clone(), p2.clone()]
            .constrained_outer_triangulation()
            .map_err(SpadeBoolopsError::TriangulationError)?
            .into_iter()
            .filter(|tri| op_pred(tri))
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>()
            .stitch_together()
            .map_err(SpadeBoolopsError::StitchError)
    }
}
