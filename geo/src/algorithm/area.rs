use crate::geometry::*;
use crate::{CoordFloat, CoordNum};
use geo_traits::{
    CoordTrait, GeometryCollectionTrait, GeometryTrait, GeometryType, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    TriangleTrait,
};

fn to_coord<T: CoordNum>(c: impl CoordTrait<T = T>) -> Coord<T> {
    Coord { x: c.x(), y: c.y() }
}

pub(crate) fn twice_signed_ring_area<T>(linestring: &impl LineStringTrait<T = T>) -> T
where
    T: CoordNum,
{
    // LineString with less than 3 points is empty, or a
    // single point, or is not closed.
    let num_coords = linestring.num_coords();
    if num_coords < 3 {
        return T::zero();
    }

    // Above test ensures the vector has at least 2 elements.
    // We check if linestring is closed, and return 0 otherwise.
    let first = to_coord(linestring.coord(0).unwrap());
    let last = to_coord(linestring.coord(num_coords - 1).unwrap());
    if first != last {
        return T::zero();
    }

    sum_line_determinants(linestring.coords().map(to_coord))
}

/// Adds up the determinants of the lines between a series of `points` shifted by the first coordinate.
///
/// Each coordinate is shifted by subtracting the first coordinate,
/// effectively translating the geometry so that the first coordinate becomes the origin.
/// This helps to reduce numerical errors when summing the determinants for area calculations.
///
/// Note that we can't use the `Centroid` trait as it requires `T: Float` and in fact computes area in the
/// implementation. Another option is to use the average of the coordinates, but it is not fool-proof to
/// divide by the length of the linestring (eg. a long line-string with T = u8)
fn sum_line_determinants<T: CoordNum>(mut points: impl Iterator<Item = Coord<T>>) -> T {
    let (Some(shift), Some(second)) = (points.next(), points.next()) else {
        return T::zero();
    };

    // NOTE: We can ignore the determinant of the first and last line (always zero after shifting)
    let mut sum = T::zero();
    let mut prev = second - shift;

    for point in points {
        let shifted = point - shift;
        let line = Line::new(prev, shifted);

        sum = sum + line.determinant();
        prev = shifted;
    }

    sum
}

/// Signed and unsigned planar area of a geometry.
///
/// This is implemented for the [`geo_types`] geometries. To compute the area of any geometry
/// implementing the [`geo_traits`] traits, use the per-geometry extension traits in this module
/// instead, e.g. [`AreaPolygon`] or [`AreaGeometry`].
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::Area;
///
/// let mut polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// assert_eq!(polygon.signed_area(), 30.);
/// assert_eq!(polygon.unsigned_area(), 30.);
///
/// polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// assert_eq!(polygon.signed_area(), -30.);
/// assert_eq!(polygon.unsigned_area(), 30.);
/// ```
pub trait Area<T>
where
    T: CoordNum,
{
    fn signed_area(&self) -> T;

    fn unsigned_area(&self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area<T>(linestring: &impl LineStringTrait<T = T>) -> T
where
    T: CoordFloat,
{
    twice_signed_ring_area(linestring) / (T::one() + T::one())
}

/// Implements [`Area`] for a `geo_types` geometry by delegating to its [`geo_traits`]
/// extension trait.
macro_rules! impl_area_via_trait {
    ($bound:ident, $geom:ident, $ext:ident) => {
        impl<T> Area<T> for $geom<T>
        where
            T: $bound,
        {
            fn signed_area(&self) -> T {
                $ext::signed_area(self)
            }

            fn unsigned_area(&self) -> T {
                $ext::unsigned_area(self)
            }
        }
    };
}

/// Defines a zero-area extension trait for a [`geo_traits`] geometry type.
macro_rules! zero_area_trait {
    ($(#[$meta:meta])* $name:ident, $geo_trait:ident) => {
        $(#[$meta])*
        pub trait $name<T>: $geo_trait<T = T>
        where
            T: CoordNum,
        {
            fn signed_area(&self) -> T {
                T::zero()
            }

            fn unsigned_area(&self) -> T {
                T::zero()
            }
        }

        impl<T: CoordNum, G: $geo_trait<T = T>> $name<T> for G {}
    };
}

zero_area_trait!(
    /// Planar area of any [`PointTrait`] implementor, which is always zero.
    AreaPoint,
    PointTrait
);
zero_area_trait!(
    /// Planar area of any [`LineStringTrait`] implementor, which is always zero.
    AreaLineString,
    LineStringTrait
);
zero_area_trait!(
    /// Planar area of any [`LineTrait`] implementor, which is always zero.
    AreaLine,
    LineTrait
);
zero_area_trait!(
    /// Planar area of any [`MultiPointTrait`] implementor, which is always zero.
    AreaMultiPoint,
    MultiPointTrait
);
zero_area_trait!(
    /// Planar area of any [`MultiLineStringTrait`] implementor, which is always zero.
    AreaMultiLineString,
    MultiLineStringTrait
);

impl_area_via_trait!(CoordNum, Point, AreaPoint);
impl_area_via_trait!(CoordNum, LineString, AreaLineString);
impl_area_via_trait!(CoordNum, Line, AreaLine);

/// Signed and unsigned planar area of any [`PolygonTrait`] implementor.
///
/// **Note.** The implementation handles polygons whose
/// holes do not all have the same orientation. The sign of
/// the output is the same as that of the exterior shell.
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::area::AreaPolygon;
///
/// let polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// assert_eq!(AreaPolygon::signed_area(&polygon), 30.);
/// ```
pub trait AreaPolygon<T>: PolygonTrait<T = T>
where
    T: CoordFloat,
{
    fn signed_area(&self) -> T {
        let area = self
            .exterior()
            .map_or(T::zero(), |exterior| get_linestring_area(&exterior));

        // We could use winding order here, but that would
        // result in computing the shoelace formula twice.
        let is_negative = area < T::zero();

        let area = self.interiors().fold(area.abs(), |total, next| {
            total - get_linestring_area(&next).abs()
        });

        if is_negative { -area } else { area }
    }

    fn unsigned_area(&self) -> T {
        AreaPolygon::signed_area(self).abs()
    }
}

impl<T: CoordFloat, G: PolygonTrait<T = T>> AreaPolygon<T> for G {}

impl_area_via_trait!(CoordFloat, Polygon, AreaPolygon);
impl_area_via_trait!(CoordNum, MultiPoint, AreaMultiPoint);
impl_area_via_trait!(CoordNum, MultiLineString, AreaMultiLineString);

/// Signed and unsigned planar area of any [`MultiPolygonTrait`] implementor.
///
/// **Note.** The implementation is a straight-forward
/// summation of the signed or unsigned areas of the individual
/// polygons.
pub trait AreaMultiPolygon<T>: MultiPolygonTrait<T = T>
where
    T: CoordFloat,
{
    fn signed_area(&self) -> T {
        self.polygons().fold(T::zero(), |total, next| {
            total + AreaPolygon::signed_area(&next)
        })
    }

    fn unsigned_area(&self) -> T {
        self.polygons().fold(T::zero(), |total, next| {
            total + AreaPolygon::unsigned_area(&next)
        })
    }
}

impl<T: CoordFloat, G: MultiPolygonTrait<T = T>> AreaMultiPolygon<T> for G {}

impl_area_via_trait!(CoordFloat, MultiPolygon, AreaMultiPolygon);

/// Planar area of any [`RectTrait`] implementor.
///
/// Because a `Rect` has no winding order, the area will always be positive.
pub trait AreaRect<T>: RectTrait<T = T>
where
    T: CoordNum,
{
    fn signed_area(&self) -> T {
        let (min, max) = (self.min(), self.max());
        (max.x() - min.x()) * (max.y() - min.y())
    }

    fn unsigned_area(&self) -> T {
        AreaRect::signed_area(self)
    }
}

impl<T: CoordNum, G: RectTrait<T = T>> AreaRect<T> for G {}

impl_area_via_trait!(CoordNum, Rect, AreaRect);

/// Signed and unsigned planar area of any [`TriangleTrait`] implementor.
pub trait AreaTriangle<T>: TriangleTrait<T = T>
where
    T: CoordFloat,
{
    fn signed_area(&self) -> T {
        sum_line_determinants(self.coords().into_iter().map(to_coord)) / (T::one() + T::one())
    }

    fn unsigned_area(&self) -> T {
        AreaTriangle::signed_area(self).abs()
    }
}

impl<T: CoordFloat, G: TriangleTrait<T = T>> AreaTriangle<T> for G {}

impl_area_via_trait!(CoordFloat, Triangle, AreaTriangle);

/// Signed and unsigned planar area of any [`GeometryTrait`] implementor.
pub trait AreaGeometry<T>: GeometryTrait<T = T>
where
    T: CoordFloat,
{
    fn signed_area(&self) -> T {
        match self.as_type() {
            GeometryType::Point(g) => AreaPoint::signed_area(g),
            GeometryType::LineString(g) => AreaLineString::signed_area(g),
            GeometryType::Polygon(g) => AreaPolygon::signed_area(g),
            GeometryType::MultiPoint(g) => AreaMultiPoint::signed_area(g),
            GeometryType::MultiLineString(g) => AreaMultiLineString::signed_area(g),
            GeometryType::MultiPolygon(g) => AreaMultiPolygon::signed_area(g),
            GeometryType::GeometryCollection(g) => AreaGeometryCollection::signed_area(g),
            GeometryType::Rect(g) => AreaRect::signed_area(g),
            GeometryType::Triangle(g) => AreaTriangle::signed_area(g),
            GeometryType::Line(g) => AreaLine::signed_area(g),
        }
    }

    fn unsigned_area(&self) -> T {
        match self.as_type() {
            GeometryType::Point(g) => AreaPoint::unsigned_area(g),
            GeometryType::LineString(g) => AreaLineString::unsigned_area(g),
            GeometryType::Polygon(g) => AreaPolygon::unsigned_area(g),
            GeometryType::MultiPoint(g) => AreaMultiPoint::unsigned_area(g),
            GeometryType::MultiLineString(g) => AreaMultiLineString::unsigned_area(g),
            GeometryType::MultiPolygon(g) => AreaMultiPolygon::unsigned_area(g),
            GeometryType::GeometryCollection(g) => AreaGeometryCollection::unsigned_area(g),
            GeometryType::Rect(g) => AreaRect::unsigned_area(g),
            GeometryType::Triangle(g) => AreaTriangle::unsigned_area(g),
            GeometryType::Line(g) => AreaLine::unsigned_area(g),
        }
    }
}

impl<T: CoordFloat, G: GeometryTrait<T = T>> AreaGeometry<T> for G {}

impl_area_via_trait!(CoordFloat, Geometry, AreaGeometry);

/// Signed and unsigned planar area of any [`GeometryCollectionTrait`] implementor.
pub trait AreaGeometryCollection<T>: GeometryCollectionTrait<T = T>
where
    T: CoordFloat,
{
    fn signed_area(&self) -> T {
        self.geometries()
            .map(|g| AreaGeometry::signed_area(&g))
            .fold(T::zero(), |acc, next| acc + next)
    }

    fn unsigned_area(&self) -> T {
        self.geometries()
            .map(|g| AreaGeometry::unsigned_area(&g))
            .fold(T::zero(), |acc, next| acc + next)
    }
}

impl<T: CoordFloat, G: GeometryCollectionTrait<T = T>> AreaGeometryCollection<T> for G {}

impl_area_via_trait!(CoordFloat, GeometryCollection, AreaGeometryCollection);

#[cfg(test)]
mod test {
    use crate::Area;
    use crate::{Line, MultiPolygon, Polygon, Rect, Triangle, coord, polygon, wkt};

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert_relative_eq!(poly.signed_area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = wkt! { POLYGON((1. 0.)) };
        assert_relative_eq!(poly.signed_area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let polygon = wkt! { POLYGON((0. 0.,5. 0.,5. 6.,0. 6.,0. 0.)) };
        assert_relative_eq!(polygon.signed_area(), 30.);
    }
    #[test]
    fn area_polygon_numerical_stability() {
        let polygon = {
            use std::f64::consts::PI;
            const NUM_VERTICES: usize = 10;
            const ANGLE_INC: f64 = 2. * PI / NUM_VERTICES as f64;

            Polygon::new(
                (0..NUM_VERTICES)
                    .map(|i| {
                        let angle = i as f64 * ANGLE_INC;
                        coord! {
                            x: angle.cos(),
                            y: angle.sin(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .into(),
                vec![],
            )
        };

        let area = polygon.signed_area();

        let shift = coord! { x: 1.5e8, y: 1.5e8 };

        use crate::map_coords::MapCoords;
        let polygon = polygon.map_coords(|c| c + shift);

        let new_area = polygon.signed_area();
        let err = (area - new_area).abs() / area;

        assert!(err < 1e-2);
    }
    #[test]
    fn rectangle_test() {
        let rect1: Rect<f32> = Rect::new(coord! { x: 10., y: 30. }, coord! { x: 20., y: 40. });
        assert_relative_eq!(rect1.signed_area(), 100.);

        let rect2: Rect<i32> = Rect::new(coord! { x: 10, y: 30 }, coord! { x: 20, y: 40 });
        assert_eq!(rect2.signed_area(), 100);
    }
    #[test]
    fn area_polygon_inner_test() {
        let poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 2., y: 1.),
                    (x: 2., y: 2.),
                    (x: 1., y: 2.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 6., y: 5.),
                    (x: 6., y: 6.),
                    (x: 5., y: 6.),
                    (x: 5., y: 5.)
                ],
            ],
        ];
        assert_relative_eq!(poly.signed_area(), 98.);
    }
    #[test]
    fn area_multipolygon_test() {
        let poly0 = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.)
        ];
        let poly1 = polygon![
            (x: 1., y: 1.),
            (x: 2., y: 1.),
            (x: 2., y: 2.),
            (x: 1., y: 2.),
            (x: 1., y: 1.)
        ];
        let poly2 = polygon![
            (x: 5., y: 5.),
            (x: 6., y: 5.),
            (x: 6., y: 6.),
            (x: 5., y: 6.),
            (x: 5., y: 5.)
        ];
        let mpoly = MultiPolygon::new(vec![poly0, poly1, poly2]);
        assert_relative_eq!(mpoly.signed_area(), 102.);
        assert_relative_eq!(mpoly.signed_area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        assert_relative_eq!(line1.signed_area(), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 0.0, y: 1.0 },
        );
        assert_relative_eq!(triangle.signed_area(), 0.5);

        let triangle = Triangle::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: 1.0 },
            coord! { x: 1.0, y: 0.0 },
        );
        // triangles are always ccw, thus positive
        assert_relative_eq!(triangle.signed_area(), 0.5);
    }
    #[test]
    fn area_triangle_numerical_stability() {
        let triangle = Triangle::<f64>::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.1, y: 10.0 },
            coord! { x: 0.0, y: 20.0 },
        );

        let area = triangle.signed_area();

        let shift = coord! { x: 1.5e8, y: 1.5e8 };

        use crate::map_coords::MapCoords;
        let triangle = triangle.map_coords(|c| c + shift);

        let new_area = triangle.signed_area();
        let err = (area - new_area).abs() / area;

        assert!(err < 1e-2);
    }
    #[test]
    fn area_triangle_same_as_polygon() {
        let shift = coord! { x: 1.5e8, y: 1.5e8 };

        use crate::map_coords::MapCoords;
        let triangle = Triangle::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.1, y: 10.0 },
            coord! { x: 0.0, y: 20.0 },
        )
        .map_coords(|c| c + shift);

        let triangle_area: f64 = triangle.signed_area();

        let polygon = triangle.to_polygon();
        let polygon_area = polygon.signed_area();

        let err = (polygon_area - triangle_area).abs() / polygon_area;

        assert!(err < 1e-2);
    }

    #[test]
    fn area_multi_polygon_area_reversed() {
        let polygon_cw: Polygon<f32> = polygon![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 0.0, y: 0.0 },
        ];
        let polygon_ccw: Polygon<f32> = polygon![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 0.0, y: 1.0 },
            coord! { x: 0.0, y: 0.0 },
        ];
        let polygon_area = polygon_cw.unsigned_area();

        let multi_polygon = MultiPolygon::new(vec![polygon_cw, polygon_ccw]);

        assert_eq!(polygon_area * 2., multi_polygon.unsigned_area());
    }

    #[test]
    fn area_north_america_cutout() {
        let poly = polygon![
            exterior: [
                (x: -102.902861858977, y: 31.6943450891131),
                (x: -102.917375513247, y: 31.6990175356827),
                (x: -102.917887344527, y: 31.7044889522597),
                (x: -102.938892711173, y: 31.7032871894594),
                (x: -102.939919687305, y: 31.7142296141915),
                (x: -102.946922353444, y: 31.713828170995),
                (x: -102.954642979004, y: 31.7210594956594),
                (x: -102.960927457803, y: 31.7130240707676),
                (x: -102.967929895872, y: 31.7126214137469),
                (x: -102.966383373178, y: 31.6962079209847),
                (x: -102.973384192133, y: 31.6958049292994),
                (x: -102.97390013779, y: 31.701276160078),
                (x: -102.980901394769, y: 31.7008727405409),
                (x: -102.987902575456, y: 31.7004689164622),
                (x: -102.986878877087, y: 31.7127206248263),
                (x: -102.976474089689, y: 31.7054378797983),
                (x: -102.975448432121, y: 31.7176893134691),
                (x: -102.96619351228, y: 31.7237224912303),
                (x: -102.976481009643, y: 31.7286309669534),
                (x: -102.976997412845, y: 31.7341016591658),
                (x: -102.978030448215, y: 31.7450427747035),
                (x: -102.985035821671, y: 31.7446391683265),
                (x: -102.985552968771, y: 31.7501095683386),
                (x: -102.992558780682, y: 31.7497055338313),
                (x: -102.993594334215, y: 31.7606460184322),
                (x: -102.973746840657, y: 31.7546100958509),
                (x: -102.966082339116, y: 31.767730116605),
                (x: -102.959074676589, y: 31.768132602064),
                (x: -102.95206693787, y: 31.7685346826851),
                (x: -102.953096767614, y: 31.7794749110023),
                (x: -102.953611796704, y: 31.7849448911322),
                (x: -102.952629078076, y: 31.7996518517642),
                (x: -102.948661251495, y: 31.8072257578725),
                (x: -102.934638176282, y: 31.8080282207231),
                (x: -102.927626524626, y: 31.8084288446215),
                (x: -102.927113253813, y: 31.8029591283411),
                (x: -102.920102042027, y: 31.8033593239799),
                (x: -102.919076759513, y: 31.792419577395),
                (x: -102.912066503301, y: 31.7928193216213),
                (x: -102.911554491357, y: 31.7873492912889),
                (x: -102.904544675025, y: 31.7877486073783),
                (x: -102.904033254331, y: 31.7822784646103),
                (x: -102.903521909259, y: 31.7768082325431),
                (x: -102.895800463718, y: 31.7695748336589),
                (x: -102.889504111843, y: 31.7776055573633),
                (x: -102.882495099915, y: 31.7780036124077),
                (x: -102.868476849997, y: 31.7787985077398),
                (x: -102.866950998738, y: 31.7623869292283),
                (x: -102.873958615171, y: 31.7619897531194),
                (x: -102.87888647278, y: 31.7688910039026),
                (x: -102.879947237315, y: 31.750650764952),
                (x: -102.886953672823, y: 31.750252825268),
                (x: -102.89396003296, y: 31.7498544807869),
                (x: -102.892939355062, y: 31.7389128078806),
                (x: -102.913954892669, y: 31.7377154844276),
                (x: -102.913443122277, y: 31.7322445829725),
                (x: -102.912931427507, y: 31.7267735918962),
                (x: -102.911908264767, y: 31.7158313407426),
                (x: -102.904905220014, y: 31.7162307607961),
                (x: -102.904394266551, y: 31.7107594775392),
                (x: -102.903372586049, y: 31.6998166417321),
                (x: -102.902861858977, y: 31.6943450891131),
            ],
            interiors: [
                [
                    (x: -102.916514879554, y: 31.7650686485918),
                    (x: -102.921022256876, y: 31.7770831833398),
                    (x: -102.93367363719, y: 31.771184865332),
                    (x: -102.916514879554, y: 31.7650686485918),
                ],
                [
                    (x: -102.935483140202, y: 31.7419852607081),
                    (x: -102.932452314332, y: 31.7328567234689),
                    (x: -102.918345099146, y: 31.7326099897391),
                    (x: -102.925566322952, y: 31.7552505533503),
                    (x: -102.928990700436, y: 31.747856686604),
                    (x: -102.935996606762, y: 31.7474559134477),
                    (x: -102.939021176592, y: 31.7539885279379),
                    (x: -102.944714388971, y: 31.7488395547293),
                    (x: -102.935996606762, y: 31.7474559134477),
                    (x: -102.935483140202, y: 31.7419852607081),
                ],
                [
                    (x: -102.956498858767, y: 31.7407805824758),
                    (x: -102.960959476367, y: 31.7475080456347),
                    (x: -102.972817445204, y: 31.742072061889),
                    (x: -102.956498858767, y: 31.7407805824758),
                ]
            ],
        ];
        // Value from shapely
        assert_relative_eq!(
            poly.unsigned_area(),
            0.006547948219252177,
            max_relative = 0.0001
        );
    }

    #[test]
    fn geo_traits_extension_traits() {
        use super::{
            AreaGeometry, AreaGeometryCollection, AreaMultiPolygon, AreaPolygon, AreaRect,
            AreaTriangle,
        };
        use crate::{Geometry, GeometryCollection, Triangle};

        let exterior =
            wkt! { POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.),(1. 1.,1. 2.,2. 2.,2. 1.,1. 1.)) };
        // Clockwise exterior gives a negative signed area.
        let reversed = wkt! { POLYGON((0. 0.,0. 10.,10. 10.,10. 0.,0. 0.)) };
        let multi = MultiPolygon::new(vec![exterior.clone(), reversed.clone()]);
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 3. });
        let triangle = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 4., y: 0. },
            coord! { x: 0., y: 3. },
        );

        // References implement the geo-traits too, and go through the blanket impls
        assert_relative_eq!(AreaPolygon::signed_area(&&exterior), 99.);
        assert_relative_eq!(AreaPolygon::signed_area(&&reversed), -100.);
        assert_relative_eq!(AreaPolygon::unsigned_area(&&reversed), 100.);
        assert_relative_eq!(AreaMultiPolygon::signed_area(&&multi), -1.);
        assert_relative_eq!(AreaMultiPolygon::unsigned_area(&&multi), 199.);
        assert_relative_eq!(AreaRect::signed_area(&&rect), 6.);
        assert_relative_eq!(AreaTriangle::signed_area(&&triangle), 6.);

        let geometries: Vec<Geometry> = vec![
            exterior.into(),
            reversed.into(),
            multi.into(),
            rect.into(),
            triangle.into(),
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }).into(),
        ];
        for geometry in &geometries {
            assert_eq!(
                AreaGeometry::signed_area(&geometry),
                Area::signed_area(geometry)
            );
            assert_eq!(
                AreaGeometry::unsigned_area(&geometry),
                Area::unsigned_area(geometry)
            );
        }

        let collection = GeometryCollection::new_from(geometries);
        assert_relative_eq!(AreaGeometryCollection::signed_area(&&collection), 10.);
        assert_relative_eq!(AreaGeometryCollection::unsigned_area(&&collection), 410.);
    }
}
