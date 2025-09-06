use std::iter::Sum;

use crate::CoordFloat;
use geo_traits_ext::*;

/// Calculation of the length
#[deprecated(
    since = "0.29.0",
    note = "Please use the `Euclidean.length(&line)` via the `Length` trait instead."
)]
pub trait EuclideanLength<T, RHS = Self> {
    /// Calculation of the length of a Line
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::EuclideanLength;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 116.34),
    /// ];
    ///
    /// assert_eq!(
    ///     2.,
    ///     line_string.euclidean_length(),
    /// )
    /// ```
    fn euclidean_length(&self) -> T;
}

#[allow(deprecated)]
impl<T, G> EuclideanLength<T> for G
where
    T: CoordFloat + Sum,
    G: GeoTraitExtWithTypeTag + EuclideanLengthTrait<T, G::Tag>,
{
    fn euclidean_length(&self) -> T {
        self.euclidean_length_trait()
    }
}

trait EuclideanLengthTrait<T, GT: GeoTypeTag>
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T;
}

#[allow(deprecated)]
impl<T, L: LineTraitExt<T = T>> EuclideanLengthTrait<T, LineTag> for L
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        let start_coord = self.start_coord();
        let end_coord = self.end_coord();
        let delta = start_coord - end_coord;
        delta.x.hypot(delta.y)
    }
}

#[allow(deprecated)]
impl<T, LS: LineStringTraitExt<T = T>> EuclideanLengthTrait<T, LineStringTag> for LS
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        let mut length = T::zero();
        for line in self.lines() {
            let start_coord = line.start_coord();
            let end_coord = line.end_coord();
            let delta = start_coord - end_coord;
            length = length + delta.x.hypot(delta.y);
        }
        length
    }
}

#[allow(deprecated)]
impl<T, MLS: MultiLineStringTraitExt<T = T>> EuclideanLengthTrait<T, MultiLineStringTag> for MLS
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        let mut length = T::zero();
        for line_string in self.line_strings_ext() {
            length = length + line_string.euclidean_length_trait();
        }
        length
    }
}

#[allow(deprecated)]
impl<T, P: PolygonTraitExt<T = T>> EuclideanLengthTrait<T, PolygonTag> for P
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Length is a 1D concept, doesn't apply to 2D polygons
        // Return zero, similar to how Area returns zero for linear geometries
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, P: PointTraitExt<T = T>> EuclideanLengthTrait<T, PointTag> for P
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // A point has no length dimension
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, MP: MultiPointTraitExt<T = T>> EuclideanLengthTrait<T, MultiPointTag> for MP
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Points have no length dimension
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, MPG: MultiPolygonTraitExt<T = T>> EuclideanLengthTrait<T, MultiPolygonTag> for MPG
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Length is a 1D concept, doesn't apply to 2D polygons
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, R: RectTraitExt<T = T>> EuclideanLengthTrait<T, RectTag> for R
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Length is a 1D concept, doesn't apply to 2D rectangles
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, TR: TriangleTraitExt<T = T>> EuclideanLengthTrait<T, TriangleTag> for TR
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Length is a 1D concept, doesn't apply to 2D triangles
        T::zero()
    }
}

#[allow(deprecated)]
impl<T, GC: GeometryCollectionTraitExt<T = T>> EuclideanLengthTrait<T, GeometryCollectionTag> for GC
where
    T: CoordFloat + Sum,
{
    fn euclidean_length_trait(&self) -> T {
        // Sum the lengths of all geometries in the collection
        // Linear geometries (lines, linestrings) will contribute their actual length
        // Non-linear geometries (points, polygons) will contribute zero
        self.geometries_ext()
            .map(|g| match g.as_type_ext() {
                GeometryTypeExt::Point(_) => T::zero(),
                GeometryTypeExt::Line(line) => line.euclidean_length_trait(),
                GeometryTypeExt::LineString(ls) => ls.euclidean_length_trait(),
                GeometryTypeExt::Polygon(_) => T::zero(),
                GeometryTypeExt::MultiPoint(_) => T::zero(),
                GeometryTypeExt::MultiLineString(mls) => mls.euclidean_length_trait(),
                GeometryTypeExt::MultiPolygon(_) => T::zero(),
                GeometryTypeExt::GeometryCollection(gc) => gc.euclidean_length_trait(),
                GeometryTypeExt::Rect(_) => T::zero(),
                GeometryTypeExt::Triangle(_) => T::zero(),
            })
            .fold(T::zero(), |acc, next| acc + next)
    }
}

#[allow(deprecated)]
impl<T, G: GeometryTraitExt<T = T>> EuclideanLengthTrait<T, GeometryTag> for G
where
    T: CoordFloat + Sum,
{
    crate::geometry_trait_ext_delegate_impl! {
        fn euclidean_length_trait(&self) -> T;
    }
}

#[cfg(test)]
mod test {
    use crate::line_string;
    #[allow(deprecated)]
    use crate::EuclideanLength;
    use crate::{coord, Line, MultiLineString};

    #[allow(deprecated)]
    #[test]
    fn empty_linestring_test() {
        let linestring = line_string![];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[allow(deprecated)]
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 0., y: 0.)];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[allow(deprecated)]
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_relative_eq!(10.0_f64, linestring.euclidean_length());
    }
    #[allow(deprecated)]
    #[test]
    fn multilinestring_test() {
        let mline = MultiLineString::new(vec![
            line_string![
                (x: 1., y: 0.),
                (x: 7., y: 0.),
                (x: 8., y: 0.),
                (x: 9., y: 0.),
                (x: 10., y: 0.),
                (x: 11., y: 0.)
            ],
            line_string![
                (x: 0., y: 0.),
                (x: 0., y: 5.)
            ],
        ]);
        assert_relative_eq!(15.0_f64, mline.euclidean_length());
    }
    #[allow(deprecated)]
    #[test]
    fn line_test() {
        let line0 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 0., y: 1. });
        let line1 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 3., y: 4. });
        assert_relative_eq!(line0.euclidean_length(), 1.);
        assert_relative_eq!(line1.euclidean_length(), 5.);
    }

    #[allow(deprecated)]
    #[test]
    fn polygon_returns_zero_test() {
        use crate::{polygon, Polygon};
        let polygon: Polygon<f64> = polygon![
            (x: 0., y: 0.),
            (x: 4., y: 0.),
            (x: 4., y: 4.),
            (x: 0., y: 4.),
            (x: 0., y: 0.),
        ];
        // Length doesn't apply to 2D polygons, should return zero
        assert_relative_eq!(polygon.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn point_returns_zero_test() {
        use crate::Point;
        let point = Point::new(3.0, 4.0);
        // Points have no length dimension
        assert_relative_eq!(point.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn comprehensive_test_scenarios() {
        use crate::{line_string, polygon};
        use crate::{
            Geometry, GeometryCollection, MultiLineString, MultiPoint, MultiPolygon, Point,
        };

        // Test cases matching the Python pytest scenarios

        // POINT EMPTY - represented as Point with NaN coordinates
        // Note: In Rust we can't easily create "empty" points, so we test regular point

        // LINESTRING EMPTY
        let empty_linestring: crate::LineString<f64> = line_string![];
        assert_relative_eq!(empty_linestring.euclidean_length(), 0.0);

        // POINT (0 0)
        let point = Point::new(0.0, 0.0);
        assert_relative_eq!(point.euclidean_length(), 0.0);

        // LINESTRING (0 0, 0 1) - length should be 1
        let linestring = line_string![(x: 0., y: 0.), (x: 0., y: 1.)];
        assert_relative_eq!(linestring.euclidean_length(), 1.0);

        // MULTIPOINT ((0 0), (1 1)) - should be 0
        let multipoint = MultiPoint::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]);
        assert_relative_eq!(multipoint.euclidean_length(), 0.0);

        // MULTILINESTRING ((0 0, 1 1), (1 1, 2 2)) - should be ~2.828427
        // Distance from (0,0) to (1,1) = sqrt(2) ≈ 1.4142135623730951
        // Distance from (1,1) to (2,2) = sqrt(2) ≈ 1.4142135623730951
        // Total ≈ 2.8284271247461903
        let multilinestring = MultiLineString::new(vec![
            line_string![(x: 0., y: 0.), (x: 1., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 2., y: 2.)],
        ]);
        assert_relative_eq!(
            multilinestring.euclidean_length(),
            2.8284271247461903,
            epsilon = 1e-10
        );

        // POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0)) - should be 0 (perimeter not included)
        let polygon = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 0.),
            (x: 1., y: 1.),
            (x: 0., y: 1.),
            (x: 0., y: 0.),
        ];
        assert_relative_eq!(polygon.euclidean_length(), 0.0);

        // MULTIPOLYGON - should be 0
        let multipolygon = MultiPolygon::new(vec![
            polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ],
            polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ],
        ]);
        assert_relative_eq!(multipolygon.euclidean_length(), 0.0);

        // GEOMETRYCOLLECTION (LINESTRING (0 0, 1 1), POLYGON (...), LINESTRING (0 0, 1 1))
        // Should sum only the linestrings: 2 * sqrt(2) ≈ 2.8284271247461903
        let collection = GeometryCollection::new_from(vec![
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]), // sqrt(2)
            Geometry::Polygon(polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ]), // contributes 0
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]), // sqrt(2)
        ]);
        // Now correctly sums only the linear geometries: 2 * sqrt(2) ≈ 2.8284271247461903
        // The polygon contributes 0 to the total
        assert_relative_eq!(
            collection.euclidean_length(),
            2.8284271247461903,
            epsilon = 1e-10
        );
    }

    // Individual test functions matching pytest parametrized scenarios

    #[allow(deprecated)]
    #[test]
    fn test_point_empty() {
        use crate::Point;
        // POINT EMPTY -> 0 (represented as empty coordinates or NaN in Rust context)
        let point = Point::new(f64::NAN, f64::NAN);
        // NaN coordinates still result in zero length for points
        assert_relative_eq!(point.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_linestring_empty() {
        // LINESTRING EMPTY -> 0
        let empty_linestring: crate::LineString<f64> = line_string![];
        assert_relative_eq!(empty_linestring.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_point_0_0() {
        use crate::Point;
        // POINT (0 0) -> 0
        let point = Point::new(0.0, 0.0);
        assert_relative_eq!(point.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_linestring_0_0_to_0_1() {
        // LINESTRING (0 0, 0 1) -> 1
        let linestring = line_string![(x: 0., y: 0.), (x: 0., y: 1.)];
        assert_relative_eq!(linestring.euclidean_length(), 1.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_multipoint() {
        // MULTIPOINT ((0 0), (1 1)) -> 0
        use crate::{MultiPoint, Point};
        let multipoint = MultiPoint::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]);
        assert_relative_eq!(multipoint.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_multilinestring_diagonal() {
        // MULTILINESTRING ((0 0, 1 1), (1 1, 2 2)) -> 2.8284271247461903
        use crate::MultiLineString;
        let multilinestring = MultiLineString::new(vec![
            line_string![(x: 0., y: 0.), (x: 1., y: 1.)], // sqrt(2)
            line_string![(x: 1., y: 1.), (x: 2., y: 2.)], // sqrt(2)
        ]);
        assert_relative_eq!(
            multilinestring.euclidean_length(),
            2.8284271247461903,
            epsilon = 1e-10
        );
    }

    #[allow(deprecated)]
    #[test]
    fn test_polygon_unit_square() {
        // POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0)) -> 0 (perimeters aren't included)
        use crate::polygon;
        let polygon = polygon![
            (x: 0., y: 0.),
            (x: 1., y: 0.),
            (x: 1., y: 1.),
            (x: 0., y: 1.),
            (x: 0., y: 0.),
        ];
        assert_relative_eq!(polygon.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_multipolygon_double_unit_squares() {
        // MULTIPOLYGON (((0 0, 1 0, 1 1, 0 1, 0 0)), ((0 0, 1 0, 1 1, 0 1, 0 0))) -> 0
        use crate::{polygon, MultiPolygon};
        let multipolygon = MultiPolygon::new(vec![
            polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ],
            polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ],
        ]);
        assert_relative_eq!(multipolygon.euclidean_length(), 0.0);
    }

    #[allow(deprecated)]
    #[test]
    fn test_geometrycollection_mixed() {
        // GEOMETRYCOLLECTION (LINESTRING (0 0, 1 1), POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0)), LINESTRING (0 0, 1 1))
        // Expected: 2.8284271247461903 (only linestrings contribute)
        use crate::{polygon, Geometry, GeometryCollection};
        let collection = GeometryCollection::new_from(vec![
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]), // sqrt(2) ≈ 1.4142135623730951
            Geometry::Polygon(polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ]), // contributes 0
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]), // sqrt(2) ≈ 1.4142135623730951
        ]);
        // Now correctly returns the expected sum of only the linear geometries
        // Expected: 2.8284271247461903 (sum of the two linestring lengths, polygon contributes 0)
        assert_relative_eq!(
            collection.euclidean_length(),
            2.8284271247461903,
            epsilon = 1e-10
        );

        // For now, let's test that individual geometries work correctly
        let linestring1 = line_string![(x: 0., y: 0.), (x: 1., y: 1.)];
        let linestring2 = line_string![(x: 0., y: 0.), (x: 1., y: 1.)];
        let expected_total = linestring1.euclidean_length() + linestring2.euclidean_length();
        assert_relative_eq!(expected_total, 2.8284271247461903, epsilon = 1e-10);
    }

    #[allow(deprecated)]
    #[test]
    fn test_geometrycollection_pytest_exact_scenario() {
        // Exact match for the Python pytest scenario:
        // GEOMETRYCOLLECTION (LINESTRING (0 0, 1 1), POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0)), LINESTRING (0 0, 1 1))
        // Expected: 2.8284271247461903
        use crate::{polygon, Geometry, GeometryCollection};

        let collection = GeometryCollection::new_from(vec![
            // LINESTRING (0 0, 1 1) - length = sqrt(2) ≈ 1.4142135623730951
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]),
            // POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0)) - contributes 0 (perimeter not included)
            Geometry::Polygon(polygon![
                (x: 0., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 1.),
                (x: 0., y: 1.),
                (x: 0., y: 0.),
            ]),
            // LINESTRING (0 0, 1 1) - length = sqrt(2) ≈ 1.4142135623730951
            Geometry::LineString(line_string![(x: 0., y: 0.), (x: 1., y: 1.)]),
        ]);

        // Total length = sqrt(2) + 0 + sqrt(2) = 2 * sqrt(2) ≈ 2.8284271247461903
        assert_relative_eq!(
            collection.euclidean_length(),
            2.8284271247461903,
            epsilon = 1e-10
        );
    }
}
