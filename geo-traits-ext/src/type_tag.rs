// Geometry type tags for dispatching algorithm traits to the corresponding implementation

pub trait GeoTypeTag {}

pub struct CoordTag;
pub struct PointTag;
pub struct LineStringTag;
pub struct PolygonTag;
pub struct MultiPointTag;
pub struct MultiLineStringTag;
pub struct MultiPolygonTag;
pub struct GeometryCollectionTag;
pub struct GeometryTag;
pub struct LineTag;
pub struct RectTag;
pub struct TriangleTag;

impl GeoTypeTag for CoordTag {}
impl GeoTypeTag for PointTag {}
impl GeoTypeTag for LineStringTag {}
impl GeoTypeTag for PolygonTag {}
impl GeoTypeTag for MultiPointTag {}
impl GeoTypeTag for MultiLineStringTag {}
impl GeoTypeTag for MultiPolygonTag {}
impl GeoTypeTag for GeometryCollectionTag {}
impl GeoTypeTag for GeometryTag {}
impl GeoTypeTag for LineTag {}
impl GeoTypeTag for RectTag {}
impl GeoTypeTag for TriangleTag {}

pub trait GeoTraitExtWithTypeTag {
    type Tag: GeoTypeTag;
}
