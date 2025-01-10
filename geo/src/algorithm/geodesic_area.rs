use crate::geometry::*;
use geographiclib_rs::{Geodesic, PolygonArea, Winding};

/// Determine the perimeter and area of a geometry on an ellipsoidal model of the earth.
///
/// This uses the geodesic measurement methods given by [Karney (2013)].
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicArea<T> {
    /// Determine the area of a geometry on an ellipsoidal model of the earth.
    ///
    /// This uses the geodesic measurement methods given by [Karney (2013)].
    ///
    /// # Assumptions
    ///  - Polygons are assumed to be wound in a counter-clockwise direction
    ///    for the exterior ring and a clockwise direction for interior rings.
    ///    This is the standard winding for geometries that follow the Simple Feature standard.
    ///    Alternative windings may result in a negative area. See "Interpreting negative area values" below.
    ///  - Polygons are assumed to be smaller than half the size of the earth. If you expect to be dealing
    ///    with polygons larger than this, please use the `unsigned` methods.
    ///
    /// # Units
    ///
    /// - return value: meter²
    ///
    /// # Interpreting negative area values
    ///
    /// A negative value can mean one of two things:
    /// 1. The winding of the polygon is in the clockwise direction (reverse winding). If this is the case, and you know the polygon is smaller than half the area of earth, you can take the absolute value of the reported area to get the correct area.
    /// 2. The polygon is larger than half the planet. In this case, the returned area of the polygon is not correct. If you expect to be dealing with very large polygons, please use the `unsigned` methods.
    ///
    /// # Examples
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::polygon;
    /// use geo::Polygon;
    ///
    /// // The O2 in London
    /// let mut polygon: Polygon<f64> = polygon![
    ///     (x: 0.00388383, y: 51.501574),
    ///     (x: 0.00538587, y: 51.502278),
    ///     (x: 0.00553607, y: 51.503299),
    ///     (x: 0.00467777, y: 51.504181),
    ///     (x: 0.00327229, y: 51.504435),
    ///     (x: 0.00187754, y: 51.504168),
    ///     (x: 0.00087976, y: 51.503380),
    ///     (x: 0.00107288, y: 51.502324),
    ///     (x: 0.00185608, y: 51.501770),
    ///     (x: 0.00388383, y: 51.501574),
    /// ];
    ///
    /// let area = polygon.geodesic_unsigned_area();
    ///
    /// assert_eq!(
    ///     78_596., // meters
    ///     area.round()
    /// );
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_signed_area(&self) -> T;

    fn geodesic_area_signed(&self) -> T; // TODO

    /// Determine the area of a geometry on an ellipsoidal model of the earth. Supports very large geometries that cover a significant portion of the earth.
    ///
    /// This uses the geodesic measurement methods given by [Karney (2013)].
    ///
    /// # Assumptions
    ///  - Polygons are assumed to be wound in a counter-clockwise direction
    ///    for the exterior ring and a clockwise direction for interior rings.
    ///    This is the standard winding for geometries that follow the Simple Features standard.
    ///    Using alternative windings will result in incorrect results.
    ///
    /// # Units
    ///
    /// - return value: meter²
    ///
    /// # Examples
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::polygon;
    /// use geo::Polygon;
    ///
    /// // Describe a polygon that covers all of the earth EXCEPT this small square.
    /// // The outside of the polygon is in this square, the inside of the polygon is the rest of the earth.
    /// let mut polygon: Polygon<f64> = polygon![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 0.0, y: 1.0),
    ///     (x: 1.0, y: 1.0),
    ///     (x: 1.0, y: 0.0),
    /// ];
    ///
    /// let area = polygon.geodesic_unsigned_area();
    ///
    /// // Over 5 trillion square meters!
    /// assert_eq!(area, 510053312945726.94);
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_unsigned_area(&self) -> T;

    fn geodesic_area_unsigned(&self) -> T; // TODO

    /// Determine the perimeter of a geometry on an ellipsoidal model of the earth.
    ///
    /// This uses the geodesic measurement methods given by [Karney (2013)].
    ///
    /// For a polygon this returns the sum of the perimeter of the exterior ring and interior rings.
    /// To get the perimeter of just the exterior ring of a polygon, do `polygon.exterior().geodesic_length()`.
    ///
    /// # Units
    ///
    /// - return value: meter
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter(&self) -> T;

    /// Determine the perimeter and area of a geometry on an ellipsoidal model of the earth, all in one operation.
    ///
    /// This returns the perimeter and area in a `(perimeter, area)` tuple and uses the geodesic measurement methods given by [Karney (2013)].
    ///
    /// # Area Assumptions
    ///  - Polygons are assumed to be wound in a counter-clockwise direction
    ///    for the exterior ring and a clockwise direction for interior rings.
    ///    This is the standard winding for Geometries that follow the Simple Features standard.
    ///    Alternative windings may result in a negative area. See "Interpreting negative area values" below.
    ///  - Polygons are assumed to be smaller than half the size of the earth. If you expect to be dealing
    ///    with polygons larger than this, please use the 'unsigned' methods.
    ///
    /// # Perimeter
    /// For a polygon this returns the sum of the perimeter of the exterior ring and interior rings.
    /// To get the perimeter of just the exterior ring of a polygon, do `polygon.exterior().geodesic_length()`.
    ///
    /// # Units
    ///
    /// - return value: (meter, meter²)
    ///
    /// # Interpreting negative area values
    ///
    /// A negative area value can mean one of two things:
    /// 1. The winding of the polygon is in the clockwise direction (reverse winding). If this is the case, and you know the polygon is smaller than half the area of earth, you can take the absolute value of the reported area to get the correct area.
    /// 2. The polygon is larger than half the planet. In this case, the returned area of the polygon is not correct. If you expect to be dealing with very large polygons, please use the 'unsigned' methods.
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter_signed_area(&self) -> (T, T);

    fn geodesic_perimeter_area_signed(&self) -> (T, T); // TODO

    /// Determine the perimeter and area of a geometry on an ellipsoidal model of the earth, all in one operation. Supports very large geometries that cover a significant portion of the earth.
    ///
    /// This returns the perimeter and area in a `(perimeter, area)` tuple and uses the geodesic measurement methods given by [Karney (2013)].
    ///
    /// # Area Assumptions
    ///  - Polygons are assumed to be wound in a counter-clockwise direction
    ///    for the exterior ring and a clockwise direction for interior rings.
    ///    This is the standard winding for Geometries that follow the Simple Features standard.
    ///    Using alternative windings will result in incorrect results.
    ///
    /// # Perimeter
    /// For a polygon this returns the perimeter of the exterior ring and interior rings.
    /// To get the perimeter of just the exterior ring of a polygon, do `polygon.exterior().geodesic_length()`.
    ///
    /// # Units
    ///
    /// - return value: (meter, meter²)
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_perimeter_unsigned_area(&self) -> (T, T);

    fn geodesic_perimeter_area_unsigned(&self) -> (T, T); // TODO
}

impl GeodesicArea<f64> for Polygon {
    fn geodesic_perimeter(&self) -> f64 {
        let (perimeter, _area) = geodesic_area(self, true, false, false);
        perimeter
    }

    fn geodesic_signed_area(&self) -> f64 {
        let (_perimeter, area) = geodesic_area(self, true, false, false);
        area
    }

    fn geodesic_area_signed(&self) -> f64 {
        // TODO
        self.geodesic_signed_area()
    }

    fn geodesic_unsigned_area(&self) -> f64 {
        let (_perimeter, area) = geodesic_area(self, false, false, false);
        area
    }

    fn geodesic_area_unsigned(&self) -> f64 {
        // TODO
        self.geodesic_unsigned_area()
    }

    fn geodesic_perimeter_signed_area(&self) -> (f64, f64) {
        geodesic_area(self, true, false, false)
    }

    fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
        // TODO
        self.geodesic_perimeter_signed_area()
    }

    fn geodesic_perimeter_unsigned_area(&self) -> (f64, f64) {
        geodesic_area(self, false, false, false)
    }

    fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
        // TODO
        self.geodesic_perimeter_unsigned_area()
    }
}

fn geodesic_area(poly: &Polygon, sign: bool, reverse: bool, exterior_only: bool) -> (f64, f64) {
    let g = Geodesic::wgs84();

    let (exterior_winding, interior_winding) = if reverse {
        (Winding::Clockwise, Winding::CounterClockwise)
    } else {
        (Winding::CounterClockwise, Winding::Clockwise)
    };

    // Add the exterior ring
    let (outer_perimeter, outer_area) = {
        let mut pa = PolygonArea::new(&g, exterior_winding);
        poly.exterior().points().for_each(|p| {
            pa.add_point(p.y(), p.x());
        });
        let (perimeter, area, _) = pa.compute(sign);
        (perimeter, area)
    };

    // Add the interior rings
    let (interior_perimeter, mut inner_area) = if exterior_only {
        (0.0, 0.0)
    } else {
        let mut inner_area = 0.;
        let mut inner_perimeter = 0.;
        poly.interiors().iter().for_each(|ring| {
            let mut pa = PolygonArea::new(&g, interior_winding);
            ring.points().for_each(|p| {
                pa.add_point(p.y(), p.x());
            });
            let (perimeter, area, _) = pa.compute(sign);
            inner_area += area.abs();
            inner_perimeter += perimeter;
        });
        (inner_perimeter, inner_area)
    };

    if outer_area < 0.0 && inner_area > 0.0 {
        inner_area = -inner_area;
    }

    (
        outer_perimeter + interior_perimeter,
        outer_area - inner_area,
    )
}

/// Generate a `GeodesicArea` implementation where the result is zero.
macro_rules! zero_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                0.0
            }

            fn geodesic_signed_area(&self) -> f64 {
                0.0
            }

            fn geodesic_area_signed(&self) -> f64 {
                // TODO
                self.geodesic_signed_area()
            }

            fn geodesic_unsigned_area(&self) -> f64 {
                0.0
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                // TODO
                self.geodesic_unsigned_area()
            }

            fn geodesic_perimeter_signed_area(&self) -> (f64, f64) {
                (0.0, 0.0)
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_signed_area()
            }

            fn geodesic_perimeter_unsigned_area(&self) -> (f64, f64) {
                (0.0, 0.0)
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_unsigned_area()
            }
        }
    };
}

/// Generate a `GeodesicArea` implementation which delegates to the `Polygon`
/// implementation.
macro_rules! to_polygon_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                self.to_polygon().geodesic_perimeter()
            }

            fn geodesic_signed_area(&self) -> f64 {
                self.to_polygon().geodesic_signed_area()
            }

            fn geodesic_area_signed(&self) -> f64 {
                // TODO
                self.geodesic_signed_area()
            }

            fn geodesic_unsigned_area(&self) -> f64 {
                self.to_polygon().geodesic_unsigned_area()
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                // TODO
                self.geodesic_unsigned_area()
            }

            fn geodesic_perimeter_signed_area(&self) -> (f64, f64) {
                self.to_polygon().geodesic_perimeter_signed_area()
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_signed_area()
            }

            fn geodesic_perimeter_unsigned_area(&self) -> (f64, f64) {
                self.to_polygon().geodesic_perimeter_unsigned_area()
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_unsigned_area()
            }
        }
    };
}

/// Generate a `GeodesicArea` implementation which calculates the area for each of its
/// sub-components and sums them up.
macro_rules! sum_impl {
    ($type:ident) => {
        impl GeodesicArea<f64> for $type {
            fn geodesic_perimeter(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_perimeter())
            }

            fn geodesic_signed_area(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_signed_area())
            }

            fn geodesic_area_signed(&self) -> f64 {
                // TODO
                self.geodesic_signed_area()
            }

            fn geodesic_unsigned_area(&self) -> f64 {
                self.iter()
                    .fold(0.0, |total, next| total + next.geodesic_unsigned_area())
            }

            fn geodesic_area_unsigned(&self) -> f64 {
                // TODO
                self.geodesic_unsigned_area()
            }

            fn geodesic_perimeter_signed_area(&self) -> (f64, f64) {
                self.iter()
                    .fold((0.0, 0.0), |(total_perimeter, total_area), next| {
                        let (perimeter, area) = next.geodesic_perimeter_signed_area();
                        (total_perimeter + perimeter, total_area + area)
                    })
            }

            fn geodesic_perimeter_area_signed(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_signed_area()
            }

            fn geodesic_perimeter_unsigned_area(&self) -> (f64, f64) {
                self.iter()
                    .fold((0.0, 0.0), |(total_perimeter, total_area), next| {
                        let (perimeter, area) = next.geodesic_perimeter_unsigned_area();
                        (total_perimeter + perimeter, total_area + area)
                    })
            }

            fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64) {
                // TODO
                self.geodesic_perimeter_unsigned_area()
            }
        }
    };
}

zero_impl!(Point);
zero_impl!(Line);
zero_impl!(LineString);
zero_impl!(MultiPoint);
zero_impl!(MultiLineString);
to_polygon_impl!(Rect);
to_polygon_impl!(Triangle);
sum_impl!(GeometryCollection);
sum_impl!(MultiPolygon);

impl GeodesicArea<f64> for Geometry<f64> {
    crate::geometry_delegate_impl! {
        fn geodesic_perimeter(&self) -> f64;
        fn geodesic_signed_area(&self) -> f64;
        #[deprecated = "This method will be removed in the future. Please use geodesic_signed_area() instead"]
        fn geodesic_area_signed(&self) -> f64;
        fn geodesic_unsigned_area(&self) -> f64;
        #[deprecated = "This method will be removed in the future. Please use geodesic_unsigned_area() instead"]
        fn geodesic_area_unsigned(&self) -> f64;
        fn geodesic_perimeter_signed_area(&self) -> (f64, f64);
        #[deprecated = "This method will be removed in the future. Please use geodesic_perimeter_signed_area() instead"]
        fn geodesic_perimeter_area_signed(&self) -> (f64, f64);
        fn geodesic_perimeter_unsigned_area(&self) -> (f64, f64);
        #[deprecated = "This method will be removed in the future. Please use geodesic_perimeter_unsigned_area() instead"]
        fn geodesic_perimeter_area_unsigned(&self) -> (f64, f64);
        fn geodesic_perimeter(&self) -> f64;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::line_measures::{Geodesic, Length};
    use crate::polygon;

    #[test]
    fn test_negative() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 144., y: -15.),
            (x: 154., y: -27.),
            (x: 148., y: -39.),
            (x: 130., y: -33.),
            (x: 117., y: -37.),
            (x: 113., y: -22.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            -7786102826806.07,
            polygon.geodesic_signed_area(),
            epsilon = 0.01
        );

        let geoid = geographiclib_rs::Geodesic::wgs84();
        assert_relative_eq!(
            geoid.area() - 7786102826806.07,
            polygon.geodesic_unsigned_area(),
            epsilon = 0.01
        );

        // Confirm that the exterior ring geodesic_length is the same as the perimeter
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_positive() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 113., y: -22.),
            (x: 117., y: -37.),
            (x: 130., y: -33.),
            (x: 148., y: -39.),
            (x: 154., y: -27.),
            (x: 144., y: -15.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_signed_area(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_unsigned_area(),
            epsilon = 0.01
        );

        // Confirm that the exterior ring geodesic_length is the same as the perimeter
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_missing_endpoint() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 113., y: -22.),
            (x: 117., y: -37.),
            (x: 130., y: -33.),
            (x: 148., y: -39.),
            (x: 154., y: -27.),
            (x: 144., y: -15.),
            // (x: 125., y: -15.), <-- missing endpoint
        ];
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_signed_area(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            7786102826806.07,
            polygon.geodesic_unsigned_area(),
            epsilon = 0.01
        );

        // Confirm that the exterior ring geodesic_length is the same as the perimeter
        assert_relative_eq!(
            polygon.exterior().length::<Geodesic>(),
            polygon.geodesic_perimeter()
        );
    }

    #[test]
    fn test_holes() {
        let mut poly = polygon![
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
                    (x: 1., y: 2.),
                    (x: 2., y: 2.),
                    (x: 2., y: 1.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 5., y: 6.),
                    (x: 6., y: 6.),
                    (x: 6., y: 5.),
                    (x: 5., y: 5.)
                ],
            ],
        ];

        assert_relative_eq!(
            1203317999173.7063,
            poly.geodesic_signed_area(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            1203317999173.7063,
            poly.geodesic_unsigned_area(),
            epsilon = 0.01
        );
        assert_relative_eq!(5307742.446635911, poly.geodesic_perimeter(), epsilon = 0.01);

        let (perimeter, area) = poly.geodesic_perimeter_signed_area();

        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);

        let (perimeter, area) = poly.geodesic_perimeter_unsigned_area();

        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);

        // Test with exterior and interior both with CW winding
        use crate::algorithm::winding_order::Winding;
        poly.exterior_mut(|exterior| {
            exterior.make_cw_winding();
        });

        let (perimeter, area) = poly.geodesic_perimeter_signed_area();
        assert_relative_eq!(-1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);

        // Test with exterior CW and interior CCW winding
        poly.interiors_mut(|interiors| {
            for interior in interiors {
                interior.make_ccw_winding();
            }
        });

        let (perimeter, area) = poly.geodesic_perimeter_signed_area();
        assert_relative_eq!(-1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);

        // Test with exterior and interior both with CCW winding
        poly.exterior_mut(|exterior| {
            exterior.make_ccw_winding();
        });

        let (perimeter, area) = poly.geodesic_perimeter_signed_area();
        assert_relative_eq!(1203317999173.7063, area, epsilon = 0.01);
        assert_relative_eq!(5307742.446635911, perimeter, epsilon = 0.01);
    }

    #[test]
    fn test_bad_interior_winding() {
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

        assert_relative_eq!(1203317999173.7063, poly.geodesic_signed_area());
    }

    #[test]
    fn test_diamond() {
        // a diamond shape
        let mut diamond = polygon![
            // exterior oriented counter-clockwise
            exterior: [
                (x: 1.0, y: 0.0),
                (x: 2.0, y: 1.0),
                (x: 1.0, y: 2.0),
                (x: 0.0, y: 1.0),
                (x: 1.0, y: 0.0),
            ],
            // interior oriented clockwise
            interiors: [
                [
                    (x: 1.0, y: 0.5),
                    (x: 0.5, y: 1.0),
                    (x: 1.0, y: 1.5),
                    (x: 1.5, y: 1.0),
                    (x: 1.0, y: 0.5),
                ],
            ],
        ];
        assert_relative_eq!(18462065880.09138, diamond.geodesic_unsigned_area());
        assert_relative_eq!(18462065880.09138, diamond.geodesic_signed_area());
        assert_relative_eq!(941333.0085011568, diamond.geodesic_perimeter());

        let (perimeter, area) = diamond.geodesic_perimeter_signed_area();
        assert_relative_eq!(941333.0085011568, perimeter);
        assert_relative_eq!(18462065880.09138, area);

        let (perimeter, area) = diamond.geodesic_perimeter_unsigned_area();
        assert_relative_eq!(941333.0085011568, perimeter);
        assert_relative_eq!(18462065880.09138, area);

        // Test with exterior and interior both with CW winding
        use crate::algorithm::winding_order::Winding;
        diamond.exterior_mut(|exterior| {
            exterior.make_cw_winding();
        });

        let (perimeter, area) = diamond.geodesic_perimeter_signed_area();
        assert_relative_eq!(-18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);

        // Test with exterior CW and interior CCW winding
        diamond.interiors_mut(|interiors| {
            for interior in interiors {
                interior.make_ccw_winding();
            }
        });

        let (perimeter, area) = diamond.geodesic_perimeter_signed_area();
        assert_relative_eq!(-18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);

        // Test with exterior and interior both with CCW winding
        diamond.exterior_mut(|exterior| {
            exterior.make_ccw_winding();
        });

        let (perimeter, area) = diamond.geodesic_perimeter_signed_area();
        assert_relative_eq!(18462065880.09138, area);
        assert_relative_eq!(941333.0085011568, perimeter);
    }

    #[test]
    fn test_very_large_polygon() {
        // Describe a polygon that covers all of the earth EXCEPT this small square.
        // The outside of the polygon is in this square, the inside of the polygon is the rest of the earth.
        let polygon_large: Polygon<f64> = polygon![
            (x: 0.0, y: 0.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 1.0),
            (x: 1.0, y: 0.0),
        ];

        let area = polygon_large.geodesic_unsigned_area();
        assert_eq!(area, 510053312945726.94);

        // A very large polygon that covers nearly all the earth, and then a hole that also covers nearly all the earth as well.
        // This is a neat polygon because signed and unsigned areas are the same, regardless of the winding order.
        let polygon_large_with_hole: Polygon<f64> = polygon![
            exterior: [
                (x: 0.5, y: 0.5),
                (x: 0.5, y: 1.0),
                (x: 1.0, y: 1.0),
                (x: 1.0, y: 0.5),
                (x: 0.5, y: 0.5),
            ],
            interiors: [
                [
                    (x: 0.0, y: 0.0),
                    (x: 2.0, y: 0.0),
                    (x: 2.0, y: 2.0),
                    (x: 0.0, y: 2.0),
                    (x: 0.0, y: 0.0),
                ],
            ],
        ];

        let area = polygon_large_with_hole.geodesic_signed_area();
        assert_relative_eq!(area, 46154562709.8, epsilon = 0.1);

        let area = polygon_large_with_hole.geodesic_unsigned_area();
        assert_relative_eq!(area, 46154562709.8, epsilon = 0.1);
    }
}
