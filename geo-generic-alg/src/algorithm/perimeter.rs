use geo_traits_ext::*;

use crate::CoordFloat;

/// Signed and unsigned planar perimeter of a geometry.
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::Perimeter;
///
/// let polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// assert_eq!(polygon.perimeter(), 22.);
///
/// let polygon_with_hole = polygon![
///     exterior: [
///         (x: 0., y: 0.),
///         (x: 10., y: 0.),
///         (x: 10., y: 10.),
///         (x: 0., y: 10.),
///         (x: 0., y: 0.),
///     ],
///     interiors: [
///         [
///             (x: 2., y: 2.),
///             (x: 8., y: 2.),
///             (x: 8., y: 8.),
///             (x: 2., y: 8.),
///             (x: 2., y: 2.),
///         ],
///     ],
/// ];
///
/// // Perimeter includes both exterior and interior rings
/// assert_eq!(polygon_with_hole.perimeter(), 40. + 24.);
/// ```
pub trait Perimeter<T>
where
    T: CoordFloat,
{
    /// Calculate the perimeter of a geometry.
    /// For polygons, this includes the perimeter of both exterior and interior rings.
    fn perimeter(&self) -> T;
}

impl<T, G> Perimeter<T> for G
where
    T: CoordFloat,
    G: GeoTraitExtWithTypeTag + PerimeterTrait<T, G::Tag>,
{
    fn perimeter(&self) -> T {
        self.perimeter_trait()
    }
}

trait PerimeterTrait<T, GT: GeoTypeTag>
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T;
}

// Helper function to calculate the perimeter of a linestring
fn linestring_perimeter<T, LS: LineStringTraitExt<T = T>>(linestring: &LS) -> T
where
    T: CoordFloat,
{
    let mut perimeter = T::zero();
    for line in linestring.lines() {
        let start_coord = line.start_coord();
        let end_coord = line.end_coord();
        let delta = start_coord - end_coord;
        perimeter = perimeter + delta.x.hypot(delta.y);
    }
    perimeter
}

impl<T, P: PointTraitExt<T = T>> PerimeterTrait<T, PointTag> for P
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        // A point has no perimeter
        T::zero()
    }
}

impl<T, L: LineTraitExt<T = T>> PerimeterTrait<T, LineTag> for L
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        // For a line, return its length as the perimeter
        let start_coord = self.start_coord();
        let end_coord = self.end_coord();
        let delta = start_coord - end_coord;
        delta.x.hypot(delta.y)
    }
}

impl<T, LS: LineStringTraitExt<T = T>> PerimeterTrait<T, LineStringTag> for LS
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        // For a linestring, return its length as the perimeter
        linestring_perimeter(self)
    }
}

impl<T, P: PolygonTraitExt<T = T>> PerimeterTrait<T, PolygonTag> for P
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        // Calculate exterior perimeter
        let mut total_perimeter = match self.exterior_ext() {
            Some(exterior) => linestring_perimeter(&exterior),
            None => T::zero(),
        };

        // Add interior perimeters
        for interior in self.interiors_ext() {
            total_perimeter = total_perimeter + linestring_perimeter(&interior);
        }

        total_perimeter
    }
}

impl<T, R: RectTraitExt<T = T>> PerimeterTrait<T, RectTag> for R
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        let width = self.width();
        let height = self.height();
        (width + height) * (T::one() + T::one())
    }
}

impl<T, TR: TriangleTraitExt<T = T>> PerimeterTrait<T, TriangleTag> for TR
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        let lines = self.to_lines();
        let mut perimeter = T::zero();
        for line in lines.iter() {
            let start_coord = line.start_coord();
            let end_coord = line.end_coord();
            let delta = start_coord - end_coord;
            perimeter = perimeter + delta.x.hypot(delta.y);
        }
        perimeter
    }
}

impl<T, MP: MultiPointTraitExt<T = T>> PerimeterTrait<T, MultiPointTag> for MP
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        // Points have no perimeter
        T::zero()
    }
}

impl<T, MLS: MultiLineStringTraitExt<T = T>> PerimeterTrait<T, MultiLineStringTag> for MLS
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        let mut total_perimeter = T::zero();
        for line_string in self.line_strings_ext() {
            total_perimeter = total_perimeter + line_string.perimeter_trait();
        }
        total_perimeter
    }
}

impl<T, MP: MultiPolygonTraitExt<T = T>> PerimeterTrait<T, MultiPolygonTag> for MP
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        let mut total_perimeter = T::zero();
        for polygon in self.polygons_ext() {
            total_perimeter = total_perimeter + polygon.perimeter_trait();
        }
        total_perimeter
    }
}

impl<T, GC: GeometryCollectionTraitExt<T = T>> PerimeterTrait<T, GeometryCollectionTag> for GC
where
    T: CoordFloat,
{
    fn perimeter_trait(&self) -> T {
        self.geometries_ext()
            .map(|g| match g.as_type_ext() {
                GeometryTypeExt::Point(_) => T::zero(),
                GeometryTypeExt::Line(line) => line.perimeter_trait(),
                GeometryTypeExt::LineString(ls) => ls.perimeter_trait(),
                GeometryTypeExt::Polygon(polygon) => polygon.perimeter_trait(),
                GeometryTypeExt::MultiPoint(_) => T::zero(),
                GeometryTypeExt::MultiLineString(mls) => mls.perimeter_trait(),
                GeometryTypeExt::MultiPolygon(mp) => mp.perimeter_trait(),
                GeometryTypeExt::GeometryCollection(gc) => gc.perimeter_trait(),
                GeometryTypeExt::Rect(rect) => rect.perimeter_trait(),
                GeometryTypeExt::Triangle(triangle) => triangle.perimeter_trait(),
            })
            .fold(T::zero(), |acc, next| acc + next)
    }
}

// Critical: GeometryTag implementation for WKB compatibility
impl<T, G: GeometryTraitExt<T = T>> PerimeterTrait<T, GeometryTag> for G
where
    T: CoordFloat,
{
    crate::geometry_trait_ext_delegate_impl! {
        fn perimeter_trait(&self) -> T;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        coord, polygon, wkt, Coord, Geometry, GeometryCollection, Line, LineString,
        MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
    };
    use approx::assert_relative_eq;

    #[test]
    fn point_perimeter() {
        let point = Point::new(1.0, 2.0);
        assert_eq!(point.perimeter(), 0.0);
    }

    #[test]
    fn multi_point_perimeter() {
        let multi_point = MultiPoint::new(vec![
            Point::new(1.0, 2.0),
            Point::new(3.0, 4.0),
            Point::new(5.0, 6.0),
        ]);
        assert_eq!(multi_point.perimeter(), 0.0);
    }

    #[test]
    fn line_perimeter() {
        let line = Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 3.0, y: 4.0 });
        assert_eq!(line.perimeter(), 5.0);
    }

    #[test]
    fn line_perimeter_degenerate() {
        // Zero-length line
        let line = Line::new(Coord { x: 1.0, y: 1.0 }, Coord { x: 1.0, y: 1.0 });
        assert_eq!(line.perimeter(), 0.0);
    }

    #[test]
    fn linestring_perimeter() {
        let linestring = LineString::from(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 3.0, y: 0.0 },
            Coord { x: 3.0, y: 4.0 },
        ]);
        assert_eq!(linestring.perimeter(), 7.0);
    }

    #[test]
    fn linestring_perimeter_empty() {
        let linestring: LineString<f64> = LineString::new(vec![]);
        assert_eq!(linestring.perimeter(), 0.0);
    }

    #[test]
    fn linestring_perimeter_single_point() {
        let linestring = LineString::from(vec![Coord { x: 1.0, y: 1.0 }]);
        assert_eq!(linestring.perimeter(), 0.0);
    }

    #[test]
    fn multi_linestring_perimeter() {
        let line1 = LineString::from(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 0.0 }]);
        let line2 = LineString::from(vec![
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 2.0, y: 1.0 },
            Coord { x: 2.0, y: 2.0 },
        ]);
        let multi_linestring = MultiLineString::new(vec![line1, line2]);
        assert_eq!(multi_linestring.perimeter(), 3.0);
    }

    #[test]
    fn polygon_perimeter() {
        // Square polygon
        let polygon = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 10.0, y: 0.0 },
                Coord { x: 10.0, y: 10.0 },
                Coord { x: 0.0, y: 10.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );
        assert_eq!(polygon.perimeter(), 40.0);
    }

    #[test]
    fn polygon_perimeter_wkt() {
        let polygon = wkt! { POLYGON((0. 0.,5. 0.,5. 6.,0. 6.,0. 0.)) };
        assert_eq!(polygon.perimeter(), 22.0);
    }

    #[test]
    fn polygon_perimeter_one_point() {
        let poly = wkt! { POLYGON((1. 0.)) };
        assert_eq!(poly.perimeter(), 0.0);
    }

    #[test]
    fn polygon_perimeter_non_square() {
        // Pentagon
        let polygon = polygon![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 3.0),
            (x: 2.0, y: 5.0),
            (x: 0.0, y: 3.0),
            (x: 0.0, y: 0.0),
        ];
        // Calculate segments:
        // (0,0) to (4,0) = 4.0
        // (4,0) to (4,3) = 3.0
        // (4,3) to (2,5) = sqrt(4 + 4) = sqrt(8) = 2.828...
        // (2,5) to (0,3) = sqrt(4 + 4) = sqrt(8) = 2.828...
        // (0,3) to (0,0) = 3.0
        let expected = 4.0 + 3.0 + (2.0_f64).hypot(2.0) + (2.0_f64).hypot(2.0) + 3.0;
        assert_relative_eq!(polygon.perimeter(), expected);
    }

    #[test]
    fn polygon_with_hole_perimeter() {
        // Square polygon with square hole
        let polygon = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 2., y: 2.),
                    (x: 8., y: 2.),
                    (x: 8., y: 8.),
                    (x: 2., y: 8.),
                    (x: 2., y: 2.),
                ],
            ],
        ];
        assert_eq!(polygon.perimeter(), 40.0 + 24.0);
    }

    #[test]
    fn polygon_with_multiple_holes_perimeter() {
        let polygon = polygon![
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
        assert_eq!(polygon.perimeter(), 40.0 + 4.0 + 4.0);
    }

    #[test]
    fn rect_perimeter() {
        let rect = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 5.0, y: 3.0 });
        assert_eq!(rect.perimeter(), 16.0);
    }

    #[test]
    fn rect_perimeter_various() {
        let rect1: Rect<f32> = Rect::new(coord! { x: 10., y: 30. }, coord! { x: 20., y: 40. });
        assert_relative_eq!(rect1.perimeter(), 40.);

        let rect2: Rect<f64> = Rect::new(coord! { x: 10., y: 30. }, coord! { x: 20., y: 40. });
        assert_eq!(rect2.perimeter(), 40.);
    }

    #[test]
    fn triangle_perimeter() {
        // 3-4-5 right triangle
        let triangle = Triangle::new(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 3.0, y: 0.0 },
            Coord { x: 0.0, y: 4.0 },
        );
        assert_eq!(triangle.perimeter(), 12.0);
    }

    #[test]
    fn triangle_perimeter_equilateral() {
        // Equilateral triangle with side length 1
        let sqrt3_2 = (3.0_f64).sqrt() / 2.0;
        let triangle = Triangle::new(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.5, y: sqrt3_2 },
        );
        assert_relative_eq!(triangle.perimeter(), 3.0, epsilon = 1e-10);
    }

    #[test]
    fn multi_polygon_perimeter() {
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
        assert_eq!(mpoly.perimeter(), 40.0 + 4.0 + 4.0);
    }

    #[test]
    fn geometry_perimeter() {
        let geom: Geometry = polygon![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.)
        ]
        .into();
        assert_eq!(geom.perimeter(), 22.0);
    }

    #[test]
    fn geometry_collection_perimeter() {
        let point = Point::new(1.0, 2.0);
        let line = Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 3.0, y: 4.0 });
        let polygon = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 0.),
            (x: 1., y: 1.),
            (x: 0., y: 1.),
            (x: 0., y: 0.)
        ];

        let collection = GeometryCollection::new_from(vec![
            Geometry::Point(point),
            Geometry::Line(line),
            Geometry::Polygon(polygon),
        ]);

        assert_eq!(collection.perimeter(), 0.0 + 5.0 + 4.0);
    }

    #[test]
    fn perimeter_numerical_stability() {
        // Test with large coordinate shifts
        let polygon = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];

        let perimeter: f64 = polygon.perimeter();

        // Shift polygon by large values
        let shift = coord! { x: 1.5e8, y: 1.5e8 };
        use crate::map_coords::MapCoords;
        let polygon = polygon.map_coords(|c| c + shift);

        let new_perimeter = polygon.perimeter();
        let err = ((perimeter - new_perimeter).abs()) / perimeter;

        assert!(err < 1e-10);
    }

    #[test]
    fn perimeter_complex_polygon() {
        // Complex polygon with irregular shape
        let polygon = polygon![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 5.0, y: 2.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 3.0),
            (x: 2.0, y: 4.0),
            (x: 1.0, y: 3.0),
            (x: 0.0, y: 2.0),
            (x: 0.0, y: 0.0),
        ];

        // Calculate expected perimeter manually
        // (0,0) to (4,0) = 4.0
        // (4,0) to (5,2) = sqrt(1 + 4) = sqrt(5)
        // (5,2) to (4,4) = sqrt(1 + 4) = sqrt(5)
        // (4,4) to (3,3) = sqrt(1 + 1) = sqrt(2)
        // (3,3) to (2,4) = sqrt(1 + 1) = sqrt(2)
        // (2,4) to (1,3) = sqrt(1 + 1) = sqrt(2)
        // (1,3) to (0,2) = sqrt(1 + 1) = sqrt(2)
        // (0,2) to (0,0) = 2.0
        let expected = 4.0
            + (1.0_f64).hypot(2.0) // to (5,2)
            + (1.0_f64).hypot(2.0) // to (4,4)
            + (1.0_f64).hypot(1.0) // to (3,3)
            + (1.0_f64).hypot(1.0) // to (2,4)
            + (1.0_f64).hypot(1.0) // to (1,3)
            + (1.0_f64).hypot(1.0) // to (0,2)
            + 2.0; // back to origin

        assert_relative_eq!(polygon.perimeter(), expected, epsilon = 1e-10);
    }

    #[test]
    fn perimeter_wkt_multipolygon() {
        let geom = wkt! {
            MULTIPOLYGON(
                ((0. 0.,5. 0.,5. 6.,0. 6.,0. 0.)),
                ((1. 1.,2. 1.,2. 2.,1. 2.,1. 1.))
            )
        };
        assert_eq!(geom.perimeter(), 22.0 + 4.0);
    }

    #[test]
    fn empty_polygon_perimeter() {
        let polygon: Polygon<f64> = Polygon::new(LineString::new(vec![]), vec![]);
        assert_eq!(polygon.perimeter(), 0.0);
    }

    #[test]
    fn mixed_geometry_types_perimeter() {
        // GeometryCollection with all types
        let collection = GeometryCollection::new_from(vec![
            Geometry::Point(Point::new(0.0, 0.0)),
            Geometry::MultiPoint(MultiPoint::new(vec![
                Point::new(1.0, 1.0),
                Point::new(2.0, 2.0),
            ])),
            Geometry::Line(Line::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 5.0, y: 0.0 },
            )),
            Geometry::LineString(LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 0.0, y: 3.0 },
            ])),
            Geometry::Polygon(polygon![
                (x: 0., y: 0.),
                (x: 2., y: 0.),
                (x: 2., y: 2.),
                (x: 0., y: 2.),
                (x: 0., y: 0.)
            ]),
            Geometry::Rect(Rect::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 3.0, y: 1.0 },
            )),
            Geometry::Triangle(Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 0.0, y: 1.0 },
            )),
        ]);

        let expected = 0.0 + 0.0 + 5.0 + 3.0 + 8.0 + 8.0 + (2.0 + (2.0_f64).sqrt());
        assert_relative_eq!(collection.perimeter(), expected);
    }
}
