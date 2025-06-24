//! Create a new geometry whose boundary is offset the specified distance from the input.

use crate::algorithm::orient::{Direction, Orient};
use crate::bool_ops::i_overlay_integration::{
    convert::{line_string_to_shape_path, multi_polygon_from_shapes, ring_to_shape_path},
    BoolOpsCoord,
};
use crate::bool_ops::{unary_union, BoolOpsNum, BooleanOps};
use crate::dimensions::{Dimensions, HasDimensions};
use crate::geometry::{
    Coord, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use i_overlay::mesh::{
    outline::offset::OutlineOffset,
    stroke::offset::StrokeOffset,
    style::{OutlineStyle, StrokeStyle},
};
// Re-export these i_overlay style types. Alternatively, we could implement our own version,
// but they'd be a 1:1 mapping, so it seems overly ceremonious.
use geo_types::coord;
pub use i_overlay::mesh::style::{LineCap, LineJoin};

/// Create a new geometry whose boundary is offset the specified distance from the input.
///
/// The buffer operation creates polygons which represent all points within a specified distance from the input geometry.
/// For example, buffering a point creates a circle, buffering a line creates a "pill" shape, and buffering a polygon
/// creates a larger polygon (or a smaller one if a negative distance is requested).
///
/// # Examples
///
/// Basic buffering with default style:
/// ```
/// use geo::{Point, Buffer, MultiPolygon};
///
/// let point = Point::new(0.0, 0.0);
/// // Creates an approximated circle with radius 2.0
/// let buffered: MultiPolygon = point.buffer(2.0);
/// ```
///
/// Default buffering rounds the point where geometries meet and where lines end.
/// Use a custom style for more control:
/// ```
/// use geo::{wkt, MultiPolygon, Buffer};
/// use geo::algorithm::buffer::{BufferStyle, LineCap, LineJoin};
///
/// let lines = wkt! {            
///     MULTILINESTRING(
///         (0. 0.,2. 0.,1. 2.),
///         (0. -1.,2. 1.,3. 3.)
///     )
/// };
/// let style = BufferStyle::new(0.5)
///     .line_cap(LineCap::Square)
///     .line_join(LineJoin::Miter(1.0));
/// let buffered: MultiPolygon = lines.buffer_with_style(style);
/// ```
pub trait Buffer {
    type Scalar: BoolOpsNum + 'static;

    /// Create a new geometry whose boundary is offset the specified distance from the input.
    ///
    /// By default, buffering uses rounded joins and end caps.
    /// See [`buffer_with_style`](Self::buffer_with_style) for more control.
    ///
    /// # Arguments
    ///
    /// * `distance` - The buffer distance. Positive values create an outward buffer,
    ///   negative values create an inward buffer (for polygons).
    ///
    /// # Returns
    ///
    /// A `MultiPolygon` representing the buffered geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Point, Buffer, MultiPolygon};
    ///
    /// let point = Point::new(5.0, 5.0);
    /// let circle: MultiPolygon = point.buffer(3.0);
    ///
    /// use geo::algorithm::Area;
    /// // Creates an approximately circular polygon centered at (5, 5) with radius 30
    /// assert_relative_eq!(circle.unsigned_area(), std::f64::consts::PI * 3.0f64.powi(2), epsilon = 2e-1);
    /// ```
    fn buffer(&self, distance: Self::Scalar) -> MultiPolygon<Self::Scalar> {
        let default_style = BufferStyle::new(distance);
        self.buffer_with_style(default_style)
    }

    /// Create a new geometry whose boundary is offset the specified distance from the input using
    /// the specific styling options where lines intersect (line joins) and end (end caps).
    /// For default (rounded) styling, see [`buffer`](Self::buffer).
    ///
    /// This method allows control over the buffer appearance through the `BufferStyle` parameter.
    ///
    /// # Arguments
    ///
    /// * `style` - A `BufferStyle` that specifies the distance, line caps, and line joins.
    ///
    /// # Returns
    ///
    /// A `MultiPolygon` representing the buffered geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{wkt, Buffer};
    /// use geo::algorithm::buffer::{BufferStyle, LineCap, LineJoin};
    ///
    /// let line_string = wkt! { LINESTRING (0. 0.,2. 0.,1. 2.) };
    /// let style = BufferStyle::new(1.5)
    ///     .line_cap(LineCap::Square)
    ///     .line_join(LineJoin::Bevel);
    /// let buffered = line_string.buffer_with_style(style);
    /// ```
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar>;
}

/// Configuration for buffer styling operations.
///
/// `BufferStyle` controls how the buffer operation creates the resulting polygon,
/// including the distance from the original geometry and how line segments are joined or capped.
///
/// # Examples
///
/// ```
/// use geo::algorithm::buffer::{BufferStyle, LineCap, LineJoin};
///
/// // Default round style
/// let round_style = BufferStyle::new(2.0);
///
/// // Square caps with mitered joins
/// let square_style = BufferStyle::new(2.0)
///     .line_cap(LineCap::Square)
///     .line_join(LineJoin::Miter(1.0));
/// ```
pub struct BufferStyle<T: BoolOpsNum> {
    distance: T,
    line_cap: LineCap<BoolOpsCoord<T>, T>,
    line_join: LineJoin<T>,
}

impl<T: BoolOpsNum> Clone for BufferStyle<T> {
    fn clone(&self) -> Self {
        Self {
            distance: self.distance,
            line_cap: self.clone_line_cap(),
            line_join: self.clone_line_join(),
        }
    }
}

impl<T: BoolOpsNum> BufferStyle<T> {
    /// Creates a new `BufferStyle` with the specified distance and default round caps/joins.
    ///
    /// # Arguments
    ///
    /// * `distance` - The buffer distance from the original geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::buffer::BufferStyle;
    ///
    /// let style = BufferStyle::new(1.5);
    /// ```
    pub fn new(distance: T) -> Self {
        BufferStyle {
            distance,
            line_cap: LineCap::Round(Self::default_join_angle()),
            line_join: LineJoin::Round(Self::default_join_angle()),
        }
    }

    /// Sets how two edges of a geometry should meet at vertices.
    ///
    /// Note: This has no effect on point geometries.
    ///
    /// # Arguments
    ///
    /// * `line_join` - The join style: `Round`, `Miter`, or `Bevel`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::buffer::{Buffer, BufferStyle, LineJoin};
    ///
    /// let style = BufferStyle::new(2.0).line_join(LineJoin::Miter(1.0));
    /// ```
    pub fn line_join(mut self, line_join: LineJoin<T>) -> Self {
        self.line_join = line_join;
        self
    }

    /// Sets how the ends of linear geometries and points should be capped.
    ///
    /// This only affects `Line`, `LineString`, `MultiLineString`, `Point`, and `MultiPoint` geometries.
    /// Two dimensional geometries, like Polygons, ignore setting.
    ///
    /// # Arguments
    ///
    /// * `line_cap` - The cap style: `Round`, `Square`, `Butt`, or `Custom`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::buffer::{Buffer, BufferStyle, LineCap};
    ///
    /// let style = BufferStyle::new(2.0).line_cap(LineCap::Square);
    /// ```
    pub fn line_cap(mut self, line_cap: LineCap<BoolOpsCoord<T>, T>) -> Self {
        self.line_cap = line_cap;
        self
    }

    // Annoyingly, i_overlay doesn't implement Clone for LineJoin
    fn clone_line_join(&self) -> LineJoin<T> {
        match self.line_join {
            LineJoin::Bevel => LineJoin::Bevel,
            LineJoin::Miter(angle) => LineJoin::Miter(angle),
            LineJoin::Round(angle) => LineJoin::Round(angle),
        }
    }

    // Annoyingly, i_overlay doesn't implement Clone for LineCap
    fn clone_line_cap(&self) -> LineCap<BoolOpsCoord<T>, T> {
        match &self.line_cap {
            LineCap::Butt => LineCap::Butt,
            LineCap::Round(angle) => LineCap::Round(*angle),
            LineCap::Square => LineCap::Square,
            LineCap::Custom(points) => LineCap::Custom(points.clone()),
        }
    }

    fn default_join_angle() -> T {
        // This is arbitrary, but the results seem to match pretty closely with JTS's own default.
        T::from_f32(0.20).expect("valid float constant")
    }

    // Used by i_overlay for buffering (Multi)Polygons
    fn outline_style(&self) -> OutlineStyle<T> {
        OutlineStyle::new(self.distance).line_join(self.clone_line_join())
    }

    // Used by i_overlay for buffering (Multi)LineStrings
    fn stroke_style(&self) -> StrokeStyle<BoolOpsCoord<T>, T> {
        // "Buffer width" is like radius, whereas "stroke width" is more like diameter, so double the
        // "stroke width" to reconcile semantics with "buffer width".
        let two = T::one() + T::one();
        StrokeStyle::new(self.distance * two)
            .line_join(self.clone_line_join())
            .end_cap(self.clone_line_cap())
            .start_cap(self.clone_line_cap())
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Geometry<F> {
    type Scalar = F;
    crate::geometry_delegate_impl! {
        fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar>;
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Point<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        if style.distance <= F::zero() {
            return MultiPolygon::new(vec![]);
        }

        match style.line_cap {
            // Without length, i_overlay can't sensible reason about orientation of template points.
            LineCap::Custom(_) => MultiPolygon::new(vec![]),
            LineCap::Butt => MultiPolygon::new(vec![]),
            LineCap::Square => {
                let a = coord!(x: self.x() - style.distance, y: self.y() - style.distance);
                let b = coord!(x: self.x() + style.distance, y: self.y() + style.distance);
                MultiPolygon(vec![Rect::new(a, b).to_polygon()])
            }
            LineCap::Round(angle) => {
                // approximate a circle
                let num_segments = (2.0 * std::f64::consts::PI / angle.to_f64()).ceil() as usize;
                let center = self.0;
                let radius = style.distance;

                let mut coords = Vec::with_capacity(num_segments + 1);
                for i in 0..num_segments {
                    let angle = F::from_f64(
                        2.0 * std::f64::consts::PI / num_segments as f64 * i as f64
                            + std::f64::consts::PI,
                    )
                    .expect("valid float constant");
                    let x = center.x + radius * angle.cos();
                    let y = center.y + radius * angle.sin();
                    coords.push(Coord { x, y });
                }
                // Close the ring
                coords.push(coords[0]);

                let polygon = Polygon::new(LineString::new(coords), vec![]);
                MultiPolygon::new(vec![polygon])
            }
        }
    }
}

impl<F: BoolOpsNum + 'static> Buffer for MultiPoint<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        if style.distance <= F::zero() {
            return MultiPolygon::new(vec![]);
        }

        if self.0.is_empty() {
            return MultiPolygon::new(vec![]);
        }

        // Buffer each point individually
        let buffered_points: Vec<MultiPolygon<F>> = self
            .0
            .iter()
            .map(|point| point.buffer_with_style(style.clone()))
            .collect();

        // Union all the buffered circles to merge overlapping ones
        unary_union(buffered_points.iter())
    }
}

impl<F: BoolOpsNum + 'static> Buffer for LineString<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        if style.distance <= F::zero() {
            return MultiPolygon::new(vec![]);
        }
        match self.dimensions() {
            Dimensions::Empty => MultiPolygon::new(vec![]),
            // Treat degenerate line like a point, similar to JTS, whereas i_overlay returns an
            // empty result
            Dimensions::ZeroDimensional => Point(self.0[0]).buffer_with_style(style),
            Dimensions::OneDimensional => {
                let subject = line_string_to_shape_path(self);
                let shapes = subject.stroke(style.stroke_style(), false);
                multi_polygon_from_shapes(shapes)
            }
            Dimensions::TwoDimensional => unreachable!("linestring can't be 2 dimensional"),
        }
    }
}

impl<F: BoolOpsNum + 'static> Buffer for MultiLineString<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        if style.distance <= F::zero() {
            return MultiPolygon::new(vec![]);
        }

        let mut degenerate_points = Vec::new();
        let mut subject = Vec::with_capacity(self.0.len());
        for line_string in self {
            match line_string.dimensions() {
                Dimensions::Empty => {}
                // Treat degenerate line like a point, similar to JTS, whereas i_overlay returns an
                // empty result
                Dimensions::ZeroDimensional => degenerate_points.push(Point(line_string[0])),
                Dimensions::OneDimensional => subject.push(line_string_to_shape_path(line_string)),
                Dimensions::TwoDimensional => unreachable!("linestring can't be 2 dimensional"),
            }
        }

        let stroked_lines = if subject.is_empty() {
            MultiPolygon::new(vec![])
        } else {
            let shapes = subject.stroke(style.stroke_style(), false);
            multi_polygon_from_shapes(shapes)
        };

        if degenerate_points.is_empty() {
            stroked_lines
        } else {
            MultiPoint(degenerate_points)
                .buffer_with_style(style)
                .union(&stroked_lines)
        }
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Polygon<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        let rewound = self.orient(Direction::Reversed);
        let subject = rewound.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.outline(style.outline_style());
        multi_polygon_from_shapes(shapes)
    }
}

impl<F: BoolOpsNum + 'static> Buffer for MultiPolygon<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        let rewound = self.orient(Direction::Reversed);
        let subject = rewound.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.outline(style.outline_style());
        multi_polygon_from_shapes(shapes)
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Line<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        if style.distance <= F::zero() {
            return MultiPolygon::new(vec![]);
        }
        match self.dimensions() {
            Dimensions::TwoDimensional => unreachable!("line can't be 2-D"),
            Dimensions::Empty => unreachable!("line can't be empty"),
            // Treat degenerate line like a point, similar to JTS, whereas i_overlay returns an
            // empty result
            Dimensions::ZeroDimensional => Point(self.start).buffer_with_style(style),
            Dimensions::OneDimensional => {
                let subject: Vec<_> = [self.start, self.end]
                    .iter()
                    .map(|c| BoolOpsCoord(*c))
                    .collect();
                let shapes = subject.stroke(style.stroke_style(), false);
                multi_polygon_from_shapes(shapes)
            }
        }
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Rect<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.to_polygon().buffer_with_style(style)
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Triangle<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.to_polygon().buffer_with_style(style)
    }
}

impl<F: BoolOpsNum + 'static> Buffer for GeometryCollection<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: BufferStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        let buffered_geometries: Vec<MultiPolygon<F>> = self
            .iter()
            .map(|geom| geom.buffer_with_style(style.clone()))
            .collect();

        // Union all the buffered results to merge overlapping geometries
        unary_union(buffered_geometries.iter())
    }
}

#[cfg(test)]
mod tests {
    // There are a lot of numeric constants in the test fixtures which equate to named constants (e.g. SQRT_2),
    // but it doesn't make sense to name them given that the fixture data is intentionally copy/pasted from an external tool
    #![allow(clippy::approx_constant)]

    use super::*;
    use crate::algorithm::Relate;
    use crate::{coord, wkt};
    use jts_test_runner::{assert_jts_tests_succeed, check_buffer_test_case};

    #[test]
    fn buffer_polygon() {
        let polygon = wkt! { POLYGON((2.0 2.0,2.0 6.0,4.0 6.0,2.0 2.0)) };
        let actual = polygon.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON ((3.7888543819998315 1.1055728090000843, 3.5861818680937785 0.7817934980757446, 3.325649726266169 0.502451068161567, 3.016761464967688 0.2777351761831863, 2.6707844932058173 0.115842850589523, 2.3003391159142073 0.0226794859072865, 1.9189381793348277 0.0016434299078536, 1.540494158905278 0.0535020210645394, 1.1788116697530158 0.1763635981188107, 0.8470839116422293 0.3657465027708164, 0.5574114167008071 0.6147425584625696, 0.3203606547097724 0.9142690619895666, 0.1445785968413555 1.253400095967856, 0.0364772975189096 1.6197650767731142, 0. 2., 0. 6., 0.0384294391935391 6.390180644032257, 0.1522409349774265 6.76536686473018, 0.3370607753949093 7.111140466039204, 0.5857864376269051 7.414213562373095, 0.8888595339607961 7.662939224605091, 1.2346331352698205 7.847759065022574, 1.6098193559677436 7.961570560806461, 2. 8., 4. 8., 4.404087761316691 7.958752940305148, 4.791508090620394 7.836713081154064, 5.14628103846132 7.638914207902305, 5.453773263881755 7.373514942482458, 5.70130161670408 7.051462224238267, 5.8786562801832645 6.686039780864038, 5.9785218959276225 6.292320213695469, 5.996779300923084 5.88654329722259, 5.932675430895927 5.485446136141952, 5.7888543819998315 5.105572809000084, 3.7888543819998315 1.1055728090000843))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_line_string() {
        let line_string = wkt! { LINESTRING(0.0 0.0, 10.0 0.0, 10.0 10.0) };
        let actual = line_string.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON ((8. 2., 8. 10., 8.03842943919354 10.390180644032258, 8.152240934977426 10.76536686473018, 8.33706077539491 11.111140466039204, 8.585786437626904 11.414213562373096, 8.888859533960796 11.66293922460509, 9.23463313526982 11.847759065022574, 9.609819355967744 11.96157056080646, 10. 12., 10.390180644032256 11.96157056080646, 10.76536686473018 11.847759065022574, 11.111140466039204 11.66293922460509, 11.414213562373096 11.414213562373096, 11.66293922460509 11.111140466039204, 11.847759065022574 10.76536686473018, 11.96157056080646 10.390180644032258, 12. 10., 12. 0., 11.96157056080646 -0.3901806440322565, 11.847759065022574 -0.7653668647301796, 11.66293922460509 -1.1111404660392044, 11.414213562373096 -1.414213562373095, 11.111140466039204 -1.6629392246050905, 10.76536686473018 -1.8477590650225735, 10.390180644032256 -1.9615705608064609, 10. -2., 0. -2., -0.3901806440322573 -1.9615705608064609, -0.7653668647301807 -1.847759065022573, -1.1111404660392044 -1.6629392246050905, -1.4142135623730954 -1.414213562373095, -1.662939224605091 -1.111140466039204, -1.8477590650225735 -0.7653668647301792, -1.9615705608064609 -0.3901806440322567, -2. 0., -1.9615705608064609 0.3901806440322572, -1.8477590650225735 0.7653668647301798, -1.6629392246050907 1.1111404660392044, -1.414213562373095 1.4142135623730951, -1.111140466039204 1.6629392246050907, -0.7653668647301795 1.8477590650225735, -0.3901806440322568 1.9615705608064609, 0. 2., 8. 2.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_linestring_closed() {
        let line_string = wkt! { LINESTRING(0.0 0.0, 5.0 0.0, 5.0 5.0, 0.0 5.0, 0.0 0.0) };
        let actual = line_string.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON (
                (-2. 0., -2. 5., -1.9615705608064609 5.390180644032257, -1.8477590650225735 5.76536686473018, -1.6629392246050907 6.111140466039204, -1.414213562373095 6.414213562373095, -1.111140466039204 6.662939224605091, -0.7653668647301795 6.847759065022574, -0.3901806440322564 6.961570560806461, 0. 7., 5. 7., 5.390180644032257 6.961570560806461, 5.765366864730179 6.847759065022574, 6.111140466039204 6.662939224605091, 6.414213562373095 6.414213562373095, 6.662939224605091 6.111140466039204, 6.847759065022574 5.765366864730179, 6.961570560806461 5.390180644032257, 7. 5., 7. 0., 6.961570560806461 -0.3901806440322565, 6.847759065022574 -0.7653668647301796, 6.662939224605091 -1.1111404660392044, 6.414213562373095 -1.414213562373095, 6.111140466039204 -1.6629392246050905, 5.765366864730179 -1.8477590650225735, 5.390180644032257 -1.9615705608064609, 5. -2., 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0.),
                (2. 2., 3. 2., 3. 3., 2. 3., 2. 2.)
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_linestring_empty() {
        let line_string = wkt! { LINESTRING EMPTY };
        let actual = line_string.buffer(2.0);
        assert_eq!(actual.0.len(), 0);
        let expected_output_from_jts = wkt! { POLYGON EMPTY };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_linestring_length_1() {
        let line_string = wkt! { LINESTRING(2.0 3.0) };

        let actual = line_string.buffer(2.0);

        let point_buffer = Point::new(2.0, 3.0).buffer(2.0);
        assert_eq!(&point_buffer, &actual);
    }

    #[test]
    fn buffer_collapsed_linestring() {
        let line_string = wkt! { LINESTRING(2.0 3.0,2.0 3.0) };

        let actual = line_string.buffer(2.0);

        let point_buffer = Point::new(2.0, 3.0).buffer(2.0);
        assert_eq!(&point_buffer, &actual);
    }

    #[test]
    fn buffer_custom_capped_line_string() {
        let line_string = wkt! { LINESTRING(0.0 0.0, 10.0 0.0, 10.0 10.0) };
        let arrow_cap = vec![
            coord!(x: -1.0, y: -2.0).into(),
            coord!(x:  3.0, y:  0.0).into(),
            coord!(x: -1.0, y:  2.0).into(),
        ];
        let style = BufferStyle::new(2.0).line_cap(LineCap::Custom(arrow_cap));
        let actual = line_string.buffer_with_style(style);
        let expected_output = wkt! {
            MULTIPOLYGON(((-6.0 0.0,2.0 -4.0,0.0 -2.0,10.0 -2.0,10.3901806473732 -1.9615705609321594,10.765366852283478 -1.8477590680122375,11.111140459775925 -1.6629392206668854,11.414213567972183 -1.4142135679721832,11.662939220666885 -1.1111404597759247,11.847759068012238 -0.7653668522834778,11.96157056093216 -0.39018064737319946,12.0 0.0,12.0 10.0,14.0 8.0,10.0 16.0,6.0 8.0,8.0 10.0,8.0 2.0,0.0 2.0,2.0 4.0,-6.0 0.0)))
        };
        check_buffer_test_case(&actual.into(), &expected_output.into()).unwrap();
    }

    #[test]
    fn buffer_empty_multi_line_string() {
        let multi_line_string = wkt! { MULTILINESTRING EMPTY };
        let actual = multi_line_string.buffer(2.0);
        let expected_output_from_jts = wkt! { POLYGON EMPTY };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_line_string_stay_separate() {
        let multi_line_string = wkt! { MULTILINESTRING ((0.0 0.0, 5.0 0.0), (10.0 0.0, 15.0 0.0)) };
        let actual = multi_line_string.buffer(2.0);
        let expected_output_from_jts = wkt! {
            MULTIPOLYGON (
                ((15. 2., 15.390180644032256 1.9615705608064609, 15.76536686473018 1.8477590650225735, 16.111140466039206 1.6629392246050905, 16.414213562373096 1.414213562373095, 16.66293922460509 1.1111404660392044, 16.847759065022572 0.7653668647301796, 16.96157056080646 0.3901806440322565, 17. 0., 16.96157056080646 -0.3901806440322565, 16.847759065022572 -0.7653668647301796, 16.66293922460509 -1.1111404660392041, 16.414213562373096 -1.414213562373095, 16.111140466039203 -1.6629392246050907, 15.76536686473018 -1.8477590650225735, 15.390180644032258 -1.9615705608064609, 15. -2., 10. -2., 9.609819355967742 -1.9615705608064609, 9.234633135269819 -1.847759065022573, 8.888859533960796 -1.6629392246050905, 8.585786437626904 -1.414213562373095, 8.337060775394908 -1.111140466039204, 8.152240934977426 -0.7653668647301792, 8.03842943919354 -0.3901806440322567, 8. 0., 8.03842943919354 0.3901806440322572, 8.152240934977426 0.7653668647301798, 8.33706077539491 1.1111404660392044, 8.585786437626904 1.4142135623730951, 8.888859533960796 1.6629392246050907, 9.23463313526982 1.8477590650225735, 9.609819355967744 1.9615705608064609, 10. 2., 15. 2.)),
                ((5. 2., 5.390180644032257 1.9615705608064609, 5.765366864730179 1.8477590650225735, 6.111140466039204 1.6629392246050905, 6.414213562373095 1.414213562373095, 6.662939224605091 1.1111404660392044, 6.847759065022574 0.7653668647301796, 6.961570560806461 0.3901806440322565, 7. 0., 6.961570560806461 -0.3901806440322565, 6.847759065022574 -0.7653668647301796, 6.662939224605091 -1.1111404660392041, 6.414213562373095 -1.414213562373095, 6.111140466039204 -1.6629392246050907, 5.765366864730179 -1.8477590650225735, 5.390180644032257 -1.9615705608064609, 5. -2., 0. -2., -0.3901806440322573 -1.9615705608064609, -0.7653668647301807 -1.847759065022573, -1.1111404660392044 -1.6629392246050905, -1.4142135623730954 -1.414213562373095, -1.662939224605091 -1.111140466039204, -1.8477590650225735 -0.7653668647301792, -1.9615705608064609 -0.3901806440322567, -2. 0., -1.9615705608064609 0.3901806440322572, -1.8477590650225735 0.7653668647301798, -1.6629392246050907 1.1111404660392044, -1.414213562373095 1.4142135623730951, -1.111140466039204 1.6629392246050907, -0.7653668647301795 1.8477590650225735, -0.3901806440322568 1.9615705608064609, 0. 2., 5. 2.))
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_line_string_with_zero_length_line_strings() {
        let multi_line_string =
            wkt! { MULTILINESTRING ((0.0 0.0, 0.0 0.0), (10.0 0.0, 10.0 15.0)) };
        let actual = multi_line_string.buffer(2.0);
        let expected_output_from_jts = wkt! {
            MULTIPOLYGON (
                ((8. 15., 8.03842943919354 15.390180644032258, 8.152240934977426 15.76536686473018, 8.33706077539491 16.111140466039203, 8.585786437626904 16.414213562373096, 8.888859533960796 16.66293922460509, 9.23463313526982 16.847759065022572, 9.609819355967744 16.96157056080646, 10. 17., 10.390180644032256 16.96157056080646, 10.76536686473018 16.847759065022572, 11.111140466039204 16.66293922460509, 11.414213562373096 16.414213562373096, 11.66293922460509 16.111140466039203, 11.847759065022574 15.76536686473018, 11.96157056080646 15.390180644032258, 12. 15., 12. 0., 11.96157056080646 -0.3901806440322565, 11.847759065022574 -0.7653668647301796, 11.66293922460509 -1.1111404660392044, 11.414213562373096 -1.414213562373095, 11.111140466039204 -1.6629392246050905, 10.76536686473018 -1.8477590650225735, 10.390180644032256 -1.9615705608064609, 10. -2., 9.609819355967744 -1.9615705608064609, 9.23463313526982 -1.8477590650225735, 8.888859533960796 -1.6629392246050907, 8.585786437626904 -1.4142135623730951, 8.33706077539491 -1.1111404660392044, 8.152240934977426 -0.7653668647301798, 8.03842943919354 -0.3901806440322572, 8. 0., 8. 15.)),
                ((2. 0., 1.9615705608064609 -0.3901806440322565, 1.8477590650225735 -0.7653668647301796, 1.6629392246050905 -1.1111404660392044, 1.4142135623730951 -1.414213562373095, 1.1111404660392046 -1.6629392246050905, 0.7653668647301797 -1.8477590650225735, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.76536686473018 1.8477590650225733, 1.1111404660392037 1.662939224605091, 1.4142135623730947 1.4142135623730954, 1.6629392246050905 1.1111404660392044, 1.847759065022573 0.7653668647301808, 1.9615705608064606 0.3901806440322574, 2. 0.))
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_line_string_elements_merge() {
        let multi_line_string = wkt! { MULTILINESTRING ((0.0 0.0, 5.0 0.0), (10.0 0.0, 15.0 0.0)) };
        let actual = multi_line_string.buffer(3.0);
        let expected_output_from_jts = wkt! {
           POLYGON ((5. 3., 5.585270966048385 2.9423558412096913, 6.14805029709527 2.77163859753386, 6.666710699058807 2.4944088369076356, 7.121320343559643 2.1213203435596424, 7.494408836907636 1.6667106990588065, 7.5 1.65625036864414, 7.5055911630923635 1.6667106990588065, 7.878679656440358 2.121320343559643, 8.333289300941194 2.494408836907636, 8.851949702904731 2.77163859753386, 9.414729033951614 2.9423558412096913, 10. 3., 15. 3., 15.585270966048386 2.9423558412096913, 16.14805029709527 2.77163859753386, 16.666710699058807 2.4944088369076356, 17.121320343559642 2.1213203435596424, 17.494408836907635 1.6667106990588065, 17.771638597533858 1.1480502970952693, 17.94235584120969 0.5852709660483848, 18. 0., 17.94235584120969 -0.5852709660483848, 17.771638597533858 -1.1480502970952693, 17.494408836907635 -1.666710699058806, 17.121320343559642 -2.1213203435596424, 16.666710699058807 -2.494408836907636, 16.14805029709527 -2.77163859753386, 15.585270966048386 -2.9423558412096913, 15. -3., 10. -3., 9.414729033951614 -2.9423558412096913, 8.85194970290473 -2.7716385975338595, 8.333289300941193 -2.4944088369076356, 7.878679656440357 -2.1213203435596424, 7.5055911630923635 -1.6667106990588059, 7.5 -1.6562503686441403, 7.4944088369076365 -1.666710699058806, 7.121320343559643 -2.1213203435596424, 6.666710699058806 -2.494408836907636, 6.14805029709527 -2.77163859753386, 5.585270966048386 -2.9423558412096913, 5. -3., 0. -3., -0.585270966048386 -2.9423558412096913, -1.148050297095271 -2.7716385975338595, -1.6667106990588065 -2.4944088369076356, -2.121320343559643 -2.1213203435596424, -2.4944088369076365 -1.6667106990588059, -2.77163859753386 -1.1480502970952688, -2.9423558412096913 -0.585270966048385, -3. 0., -2.9423558412096913 0.5852709660483858, -2.77163859753386 1.1480502970952697, -2.494408836907636 1.6667106990588065, -2.1213203435596424 2.121320343559643, -1.6667106990588059 2.494408836907636, -1.1480502970952693 2.77163859753386, -0.5852709660483852 2.9423558412096913, 0. 3., 5. 3.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_polygon_elements_stay_separate() {
        let multi_polygon = wkt! { MULTIPOLYGON (
            ((20. 40., 50. 40., 50. 30., 20. 30., 20. 40.)),
            ((0. 40., 10. 40., 10. 20., 0. 20., 0. 40.))
        )};
        let actual = multi_polygon.buffer(2.);
        let expected_output_from_jts = wkt! {
           MULTIPOLYGON (
               ((18. 40., 18.03842943919354 40.390180644032256, 18.152240934977428 40.76536686473018, 18.33706077539491 41.1111404660392, 18.585786437626904 41.41421356237309, 18.888859533960797 41.66293922460509, 19.23463313526982 41.84775906502257, 19.609819355967744 41.961570560806464, 20. 42., 50. 42., 50.390180644032256 41.961570560806464, 50.76536686473018 41.84775906502257, 51.1111404660392 41.66293922460509, 51.41421356237309 41.41421356237309, 51.66293922460509 41.1111404660392, 51.84775906502257 40.76536686473018, 51.961570560806464 40.390180644032256, 52. 40., 52. 30., 51.961570560806464 29.609819355967744, 51.84775906502257 29.23463313526982, 51.66293922460509 28.888859533960797, 51.41421356237309 28.585786437626904, 51.1111404660392 28.33706077539491, 50.76536686473018 28.152240934977428, 50.390180644032256 28.03842943919354, 50. 28., 20. 28., 19.609819355967744 28.03842943919354, 19.23463313526982 28.152240934977428, 18.888859533960797 28.33706077539491, 18.585786437626904 28.585786437626904, 18.33706077539491 28.888859533960797, 18.152240934977428 29.23463313526982, 18.03842943919354 29.609819355967744, 18. 30., 18. 40.)),
               ((-2. 40., -1.9615705608064609 40.390180644032256, -1.8477590650225735 40.76536686473018, -1.6629392246050907 41.1111404660392, -1.414213562373095 41.41421356237309, -1.111140466039204 41.66293922460509, -0.7653668647301795 41.84775906502257, -0.3901806440322564 41.961570560806464, 0. 42., 10. 42., 10.390180644032256 41.961570560806464, 10.76536686473018 41.84775906502257, 11.111140466039204 41.66293922460509, 11.414213562373096 41.41421356237309, 11.66293922460509 41.1111404660392, 11.847759065022574 40.76536686473018, 11.96157056080646 40.390180644032256, 12. 40., 12. 20., 11.96157056080646 19.609819355967744, 11.847759065022574 19.23463313526982, 11.66293922460509 18.888859533960797, 11.414213562373096 18.585786437626904, 11.111140466039204 18.33706077539491, 10.76536686473018 18.152240934977428, 10.390180644032256 18.03842943919354, 10. 18., 0. 18., -0.3901806440322573 18.03842943919354, -0.7653668647301807 18.152240934977428, -1.1111404660392044 18.33706077539491, -1.4142135623730954 18.585786437626904, -1.662939224605091 18.888859533960797, -1.8477590650225735 19.23463313526982, -1.9615705608064609 19.609819355967744, -2. 20., -2. 40.))
           )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_polygon_elements_merge() {
        let multi_polygon = wkt! { MULTIPOLYGON (
            ((20. 40., 50. 40., 50. 30., 20. 30., 20. 40.)),
            ((0. 40., 10. 40., 10. 20., 0. 20., 0. 40.))
        )};
        let actual = multi_polygon.buffer(5.);
        let expected_output_from_jts = wkt! {
            POLYGON ((-5. 40., -4.903926402016152 40.97545161008064, -4.619397662556434 41.91341716182545, -4.157348061512726 42.777851165098014, -3.5355339059327373 43.53553390593274, -2.77785116509801 44.15734806151273, -1.9134171618254485 44.61939766255644, -0.975451610080641 44.903926402016154, 0. 45., 10. 45., 10.975451610080642 44.903926402016154, 11.91341716182545 44.61939766255644, 12.777851165098012 44.15734806151273, 13.535533905932738 43.53553390593274, 14.157348061512726 42.777851165098014, 14.619397662556434 41.91341716182545, 14.903926402016152 40.97545161008064, 15. 40., 15.096073597983848 40.97545161008064, 15.380602337443566 41.91341716182545, 15.842651938487274 42.777851165098014, 16.464466094067262 43.53553390593274, 17.22214883490199 44.15734806151273, 18.08658283817455 44.61939766255644, 19.02454838991936 44.903926402016154, 20. 45., 50. 45., 50.97545161008064 44.903926402016154, 51.91341716182545 44.61939766255644, 52.777851165098014 44.15734806151273, 53.53553390593274 43.53553390593274, 54.15734806151273 42.777851165098014, 54.61939766255644 41.91341716182545, 54.903926402016154 40.97545161008064, 55. 40., 55. 30., 54.903926402016154 29.02454838991936, 54.61939766255644 28.08658283817455, 54.15734806151273 27.22214883490199, 53.53553390593274 26.464466094067262, 52.777851165098014 25.842651938487272, 51.91341716182545 25.380602337443566, 50.97545161008064 25.096073597983846, 50. 25., 20. 25., 19.024548389919357 25.096073597983846, 18.08658283817455 25.38060233744357, 17.22214883490199 25.842651938487272, 16.464466094067262 26.464466094067262, 15.842651938487272 27.22214883490199, 15.380602337443566 28.08658283817455, 15.096073597983848 29.024548389919357, 15. 30., 15. 20., 14.903926402016152 19.02454838991936, 14.619397662556434 18.08658283817455, 14.157348061512726 17.22214883490199, 13.535533905932738 16.464466094067262, 12.777851165098012 15.842651938487274, 11.91341716182545 15.380602337443566, 10.975451610080642 15.096073597983848, 10. 15., 0. 15., -0.9754516100806433 15.096073597983848, -1.9134171618254516 15.380602337443568, -2.777851165098011 15.842651938487274, -3.5355339059327386 16.464466094067262, -4.157348061512727 17.22214883490199, -4.619397662556434 18.08658283817455, -4.903926402016152 19.024548389919357, -5. 20., -5. 40.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_rect() {
        let rect = Rect::new((0.0, 0.0), (10.0, 5.0));
        let actual = rect.buffer(2.0);
        // First converted to Poly to use as JTS test runner input
        // println!("{:?}", rect.to_polygon());
        let expected_output_from_jts = wkt! {
            POLYGON ((10. -2., 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -2. 5., -1.9615705608064609 5.390180644032257, -1.8477590650225735 5.76536686473018, -1.6629392246050907 6.111140466039204, -1.414213562373095 6.414213562373095, -1.111140466039204 6.662939224605091, -0.7653668647301795 6.847759065022574, -0.3901806440322564 6.961570560806461, 0. 7., 10. 7., 10.390180644032256 6.961570560806461, 10.76536686473018 6.847759065022574, 11.111140466039204 6.662939224605091, 11.414213562373096 6.414213562373095, 11.66293922460509 6.111140466039204, 11.847759065022574 5.765366864730179, 11.96157056080646 5.390180644032257, 12. 5., 12. 0., 11.96157056080646 -0.3901806440322565, 11.847759065022574 -0.7653668647301796, 11.66293922460509 -1.1111404660392044, 11.414213562373096 -1.414213562373095, 11.111140466039204 -1.6629392246050905, 10.76536686473018 -1.8477590650225735, 10.390180644032256 -1.9615705608064609, 10. -2.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_triangle() {
        let triangle: Triangle = Triangle::new(
            coord!(x: 0.0, y: 0.0),
            coord!(x: 5.0, y: 0.0),
            coord!(x: 2.5, y: 4.0),
        );
        // First converted to Poly to use as JTS test runner input
        // println!("{:?}", triangle.to_polygon());
        let actual = triangle.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON ((-1.695996608010176 1.05999788000636, 0.804003391989824 5.05999788000636, 1.0517599500179284 5.379347946541381, 1.358665447443173 5.642362761127975, 1.7121852670464854 5.838300287369118, 2.0978809720606573 5.959158055739536, 2.5 6., 2.902119027939343 5.959158055739536, 3.287814732953515 5.838300287369118, 3.641334552556827 5.642362761127975, 3.948240049982071 5.379347946541382, 4.1959966080101765 5.05999788000636, 6.6959966080101765 1.05999788000636, 6.868234748006532 0.713932018010797, 6.9706814490978655 0.3411958765599848, 6.999509621199342 -0.0442862815244382, 6.953642333473384 -0.4281140418868183, 6.834793045115888 -0.7959487933242723, 6.647401595850507 -1.1340493736999298, 6.398468347327018 -1.4297853970174819, 6.097292671385227 -1.6721090853542637, 5.755125554449403 -1.851967979479527, 5.384749295672485 -1.9626431105729658, 5. -2., 0. -2., -0.3847492956724843 -1.9626431105729658, -0.7551255544494029 -1.851967979479527, -1.0972926713852271 -1.6721090853542635, -1.3984683473270176 -1.4297853970174825, -1.6474015958505068 -1.13404937369993, -1.8347930451158885 -0.7959487933242725, -1.9536423334733846 -0.428114041886819, -1.9995096211993426 -0.044286281524438, -1.9706814490978655 0.3411958765599851, -1.8682347480065324 0.7139320180107968, -1.695996608010176 1.05999788000636))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_collapsed_line() {
        let line = Line::new(coord!(x: 2.0, y: 3.0), coord!(x: 2.0, y: 3.0));
        let actual = line.buffer(2.0);

        let point_buffer = Point::new(2.0, 3.0).buffer(2.0);
        assert_eq!(&point_buffer, &actual);
    }

    #[test]
    fn buffer_line() {
        let line = Line::new(coord!(x: 0.0, y: 0.0), coord!(x: 5.0, y: 0.0));
        let actual = line.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON ((5. 2., 5.390180644032257 1.9615705608064609, 5.765366864730179 1.8477590650225735, 6.111140466039204 1.6629392246050905, 6.414213562373095 1.414213562373095, 6.662939224605091 1.1111404660392044, 6.847759065022574 0.7653668647301796, 6.961570560806461 0.3901806440322565, 7. 0., 6.961570560806461 -0.3901806440322565, 6.847759065022574 -0.7653668647301796, 6.662939224605091 -1.1111404660392041, 6.414213562373095 -1.414213562373095, 6.111140466039204 -1.6629392246050907, 5.765366864730179 -1.8477590650225735, 5.390180644032257 -1.9615705608064609, 5. -2., 0. -2., -0.3901806440322573 -1.9615705608064609, -0.7653668647301807 -1.847759065022573, -1.1111404660392044 -1.6629392246050905, -1.4142135623730954 -1.414213562373095, -1.662939224605091 -1.111140466039204, -1.8477590650225735 -0.7653668647301792, -1.9615705608064609 -0.3901806440322567, -2. 0., -1.9615705608064609 0.3901806440322572, -1.8477590650225735 0.7653668647301798, -1.6629392246050907 1.1111404660392044, -1.414213562373095 1.4142135623730951, -1.111140466039204 1.6629392246050907, -0.7653668647301795 1.8477590650225735, -0.3901806440322568 1.9615705608064609, 0. 2., 5. 2.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_point() {
        let point = Point::new(2.0, 3.0);
        let actual = point.buffer(2.0);
        let expected_output_from_jts = wkt! {
            POLYGON ((4. 3., 3.961570560806461 2.6098193559677436, 3.8477590650225735 2.2346331352698203, 3.6629392246050907 1.8888595339607956, 3.414213562373095 1.585786437626905, 3.1111404660392044 1.3370607753949095, 2.7653668647301797 1.1522409349774265, 2.390180644032257 1.0384294391935391, 2. 1., 1.6098193559677436 1.0384294391935391, 1.2346331352698205 1.1522409349774265, 0.8888595339607961 1.3370607753949093, 0.5857864376269051 1.5857864376269049, 0.3370607753949093 1.8888595339607956, 0.1522409349774265 2.2346331352698203, 0.0384294391935391 2.6098193559677427, 0. 3., 0.0384294391935391 3.390180644032257, 0.1522409349774265 3.7653668647301792, 0.3370607753949091 4.111140466039204, 0.5857864376269046 4.414213562373095, 0.8888595339607956 4.662939224605091, 1.2346331352698194 4.847759065022573, 1.6098193559677427 4.961570560806461, 2. 5., 2.390180644032257 4.961570560806461, 2.76536686473018 4.847759065022573, 3.1111404660392035 4.662939224605091, 3.414213562373095 4.414213562373096, 3.6629392246050907 4.111140466039204, 3.847759065022573 3.765366864730181, 3.961570560806461 3.3901806440322573, 4. 3.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_square_capped_point() {
        let point = Point::new(0.0, 0.0);
        let actual = point.buffer_with_style(BufferStyle::new(2.0).line_cap(LineCap::Square));
        let expected_output_from_jts = wkt! {
            POLYGON((2. 2.,2. -2.,-2. -2.,-2. 2.,2. 2.))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_butt_capped_point() {
        let point = Point::new(0.0, 0.0);
        let actual = point.buffer_with_style(BufferStyle::new(2.0).line_cap(LineCap::Butt));
        let expected_output_from_jts = wkt! {
            POLYGON EMPTY
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_custom_capped_point() {
        let point = Point::new(0.0, 0.0);
        let arrow_cap = vec![
            coord!(x: -1.0, y: -2.0).into(),
            coord!(x:  3.0, y:  0.0).into(),
            coord!(x: -1.0, y:  2.0).into(),
        ];
        let style = BufferStyle::new(2.0).line_cap(LineCap::Custom(arrow_cap));
        let actual = point.buffer_with_style(style);
        let expected_output = wkt! {
            POLYGON EMPTY
        };
        check_buffer_test_case(&actual.into(), &expected_output.into()).unwrap();
    }

    #[test]
    fn buffer_multi_point_separate() {
        let multi_point = wkt! { MULTIPOINT (0.0 0.0, 10.0 0.0) };
        let actual = multi_point.buffer(2.0);
        // Should result in two separate circles
        assert_eq!(actual.0.len(), 2);

        let expected_output_from_jts = wkt! {
            MULTIPOLYGON (
                ((12. 0., 11.96157056080646 -0.3901806440322565, 11.847759065022574 -0.7653668647301796, 11.66293922460509 -1.1111404660392044, 11.414213562373096 -1.414213562373095, 11.111140466039204 -1.6629392246050905, 10.76536686473018 -1.8477590650225735, 10.390180644032256 -1.9615705608064609, 10. -2., 9.609819355967744 -1.9615705608064609, 9.23463313526982 -1.8477590650225735, 8.888859533960796 -1.6629392246050907, 8.585786437626904 -1.4142135623730951, 8.33706077539491 -1.1111404660392044, 8.152240934977426 -0.7653668647301798, 8.03842943919354 -0.3901806440322572, 8. 0., 8.03842943919354 0.3901806440322567, 8.152240934977426 0.7653668647301792, 8.337060775394908 1.111140466039204, 8.585786437626904 1.414213562373095, 8.888859533960796 1.6629392246050905, 9.234633135269819 1.847759065022573, 9.609819355967742 1.9615705608064609, 10. 2., 10.390180644032256 1.9615705608064609, 10.76536686473018 1.8477590650225733, 11.111140466039204 1.662939224605091, 11.414213562373094 1.4142135623730954, 11.66293922460509 1.1111404660392044, 11.847759065022572 0.7653668647301808, 11.96157056080646 0.3901806440322574, 12. 0.)),
                ((2. 0., 1.9615705608064609 -0.3901806440322565, 1.8477590650225735 -0.7653668647301796, 1.6629392246050905 -1.1111404660392044, 1.4142135623730951 -1.414213562373095, 1.1111404660392046 -1.6629392246050905, 0.7653668647301797 -1.8477590650225735, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.76536686473018 1.8477590650225733, 1.1111404660392037 1.662939224605091, 1.4142135623730947 1.4142135623730954, 1.6629392246050905 1.1111404660392044, 1.847759065022573 0.7653668647301808, 1.9615705608064606 0.3901806440322574, 2. 0.))
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_point_overlapping() {
        let multi_point = wkt! { MULTIPOINT (0.0 0.0, 2.0 0.0) };
        let actual = multi_point.buffer(2.0);
        // Should result in one merged polygon since circles overlap
        assert_eq!(actual.0.len(), 1);
        let expected_output_from_jts = wkt! {
            POLYGON ((1. 1.722345041357806, 1.2346331352698194 1.847759065022573, 1.6098193559677427 1.9615705608064609, 2. 2., 2.390180644032257 1.9615705608064609, 2.76536686473018 1.8477590650225733, 3.1111404660392035 1.662939224605091, 3.414213562373095 1.4142135623730954, 3.6629392246050907 1.1111404660392044, 3.847759065022573 0.7653668647301808, 3.961570560806461 0.3901806440322574, 4. 0., 3.961570560806461 -0.3901806440322565, 3.8477590650225735 -0.7653668647301796, 3.6629392246050907 -1.1111404660392044, 3.414213562373095 -1.414213562373095, 3.1111404660392044 -1.6629392246050905, 2.7653668647301797 -1.8477590650225735, 2.390180644032257 -1.9615705608064609, 2. -2., 1.6098193559677436 -1.9615705608064609, 1.2346331352698205 -1.8477590650225735, 1.0000000000000002 -1.722345041357806, 0.7653668647301797 -1.8477590650225735, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.76536686473018 1.8477590650225733, 1. 1.722345041357806))
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_multi_point_empty() {
        let multi_point = wkt! { MULTIPOINT EMPTY };
        let actual = multi_point.buffer(2.0);
        assert_eq!(actual.0.len(), 0);
    }

    #[test]
    fn buffer_multi_point_zero_distance() {
        let multi_point = wkt! { MULTIPOINT (0.0 0.0,5.0 0.0) };
        let actual = multi_point.buffer(0.0);
        assert_eq!(actual.0.len(), 0);
    }

    #[test]
    fn buffer_multi_point_single_point() {
        let multi_point = wkt! { MULTIPOINT (2.0 3.0) };
        let actual = multi_point.buffer(2.0);

        let single_point_buffer = Point::new(2.0, 3.0).buffer(2.);
        check_buffer_test_case(&actual.into(), &single_point_buffer.into()).unwrap();
    }

    #[test]
    fn buffer_geometry_collection_separate_geometries() {
        let geometry_collection = wkt! {
            GEOMETRYCOLLECTION(
                POINT(0.0 0.0),
                LINESTRING(10.0 0.0, 15.0 0.0)
            )
        };
        let actual = geometry_collection.buffer(2.0);
        // Should result in two separate buffered geometries
        assert_eq!(actual.0.len(), 2);
        let expected_output_from_jts = wkt! {
            MULTIPOLYGON (
                ((15. 2., 15.390180644032256 1.9615705608064609, 15.76536686473018 1.8477590650225735, 16.111140466039206 1.6629392246050905, 16.414213562373096 1.414213562373095, 16.66293922460509 1.1111404660392044, 16.847759065022572 0.7653668647301796, 16.96157056080646 0.3901806440322565, 17. 0., 16.96157056080646 -0.3901806440322565, 16.847759065022572 -0.7653668647301796, 16.66293922460509 -1.1111404660392041, 16.414213562373096 -1.414213562373095, 16.111140466039203 -1.6629392246050907, 15.76536686473018 -1.8477590650225735, 15.390180644032258 -1.9615705608064609, 15. -2., 10. -2., 9.609819355967742 -1.9615705608064609, 9.234633135269819 -1.847759065022573, 8.888859533960796 -1.6629392246050905, 8.585786437626904 -1.414213562373095, 8.337060775394908 -1.111140466039204, 8.152240934977426 -0.7653668647301792, 8.03842943919354 -0.3901806440322567, 8. 0., 8.03842943919354 0.3901806440322572, 8.152240934977426 0.7653668647301798, 8.33706077539491 1.1111404660392044, 8.585786437626904 1.4142135623730951, 8.888859533960796 1.6629392246050907, 9.23463313526982 1.8477590650225735, 9.609819355967744 1.9615705608064609, 10. 2., 15. 2.)),
                ((2. 0., 1.9615705608064609 -0.3901806440322565, 1.8477590650225735 -0.7653668647301796, 1.6629392246050905 -1.1111404660392044, 1.4142135623730951 -1.414213562373095, 1.1111404660392046 -1.6629392246050905, 0.7653668647301797 -1.8477590650225735, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.76536686473018 1.8477590650225733, 1.1111404660392037 1.662939224605091, 1.4142135623730947 1.4142135623730954, 1.6629392246050905 1.1111404660392044, 1.847759065022573 0.7653668647301808, 1.9615705608064606 0.3901806440322574, 2. 0.))
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    #[test]
    fn buffer_geometry_collection_overlapping_geometries() {
        let geometry_collection = wkt! {
            GEOMETRYCOLLECTION(
                POINT(0.0 0.0),
                LINESTRING(1.0 0.0, 3.0 0.0)
            )
        };
        let actual = geometry_collection.buffer(2.0);
        assert_eq!(actual.0.len(), 1);

        let expected_output_from_jts = wkt! {
            POLYGON ((0.4999999999999998 -1.9282572233777517, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.4999999999999999 1.9282572233777517, 0.6098193559677432 1.9615705608064609, 1. 2., 3. 2., 3.390180644032257 1.9615705608064609, 3.7653668647301797 1.8477590650225735, 4.111140466039204 1.6629392246050905, 4.414213562373095 1.414213562373095, 4.662939224605091 1.1111404660392044, 4.847759065022574 0.7653668647301796, 4.961570560806461 0.3901806440322565, 5. 0., 4.961570560806461 -0.3901806440322565, 4.847759065022574 -0.7653668647301796, 4.662939224605091 -1.1111404660392041, 4.414213562373095 -1.414213562373095, 4.111140466039204 -1.6629392246050907, 3.7653668647301797 -1.8477590650225735, 3.390180644032257 -1.9615705608064609, 3. -2., 1. -2., 0.6098193559677427 -1.9615705608064609, 0.4999999999999998 -1.9282572233777517))
        };

        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap()
    }

    #[test]
    fn buffer_geometry_collection_empty() {
        let geometry_collection = wkt! { GEOMETRYCOLLECTION EMPTY };
        let actual = geometry_collection.buffer(2.0);
        assert_eq!(actual.0.len(), 0);
    }

    #[test]
    fn buffer_geometry_collection_zero_distance() {
        let point = Point::new(0.0, 0.0);
        let polygon = wkt! { POLYGON((1.0 1.0, 2.0 1.0, 2.0 2.0, 1.0 2.0, 1.0 1.0)) };
        let geometry_collection =
            GeometryCollection::new_from(vec![point.into(), polygon.clone().into()]);
        let actual = geometry_collection.buffer(0.0);

        // Point coalesces to nothing, but the original Polygon should be maintained.
        assert_eq!(actual.0.len(), 1);
        assert!(actual.0[0].relate(&polygon).is_equal_topo());
    }

    #[test]
    fn buffer_geometry_collection_mixed_types() {
        let geometry_collection = wkt! {
            GEOMETRYCOLLECTION(
                POINT(0.0 0.0),
                POLYGON((10.0 10.0, 12.0 10.0, 12.0 12.0, 10.0 12.0, 10.0 10.0)),
                LINESTRING(20.0 0.0, 25.0 0.0)
            )
        };
        let actual = geometry_collection.buffer(2.0);

        // Should result in three separate buffered polygons since they don't overlap
        assert_eq!(actual.0.len(), 3);

        let expected_output_from_jts = wkt! {
            MULTIPOLYGON (
                ((25. 2., 25.390180644032256 1.9615705608064609, 25.76536686473018 1.8477590650225735, 26.111140466039206 1.6629392246050905, 26.414213562373096 1.414213562373095, 26.66293922460509 1.1111404660392044, 26.847759065022572 0.7653668647301796, 26.96157056080646 0.3901806440322565, 27. 0., 26.96157056080646 -0.3901806440322565, 26.847759065022572 -0.7653668647301796, 26.66293922460509 -1.1111404660392041, 26.414213562373096 -1.414213562373095, 26.111140466039203 -1.6629392246050907, 25.76536686473018 -1.8477590650225735, 25.390180644032256 -1.9615705608064609, 25. -2., 20. -2., 19.609819355967744 -1.9615705608064609, 19.23463313526982 -1.847759065022573, 18.888859533960797 -1.6629392246050905, 18.585786437626904 -1.414213562373095, 18.33706077539491 -1.111140466039204, 18.152240934977428 -0.7653668647301792, 18.03842943919354 -0.3901806440322567, 18. 0., 18.03842943919354 0.3901806440322572, 18.152240934977428 0.7653668647301798, 18.33706077539491 1.1111404660392044, 18.585786437626904 1.4142135623730951, 18.888859533960797 1.6629392246050907, 19.23463313526982 1.8477590650225735, 19.609819355967744 1.9615705608064609, 20. 2., 25. 2.)),
                ((8. 10., 8. 12., 8.03842943919354 12.390180644032258, 8.152240934977426 12.76536686473018, 8.33706077539491 13.111140466039204, 8.585786437626904 13.414213562373096, 8.888859533960796 13.66293922460509, 9.23463313526982 13.847759065022574, 9.609819355967744 13.96157056080646, 10. 14., 12. 14., 12.390180644032256 13.96157056080646, 12.76536686473018 13.847759065022574, 13.111140466039204 13.66293922460509, 13.414213562373096 13.414213562373096, 13.66293922460509 13.111140466039204, 13.847759065022574 12.76536686473018, 13.96157056080646 12.390180644032256, 14. 12., 14. 10., 13.96157056080646 9.609819355967744, 13.847759065022574 9.23463313526982, 13.66293922460509 8.888859533960796, 13.414213562373096 8.585786437626904, 13.111140466039204 8.33706077539491, 12.76536686473018 8.152240934977426, 12.390180644032256 8.03842943919354, 12. 8., 10. 8., 9.609819355967744 8.03842943919354, 9.23463313526982 8.152240934977426, 8.888859533960796 8.33706077539491, 8.585786437626904 8.585786437626904, 8.33706077539491 8.888859533960796, 8.152240934977426 9.23463313526982, 8.03842943919354 9.609819355967742, 8. 10.)),
                ((2. 0., 1.9615705608064609 -0.3901806440322565, 1.8477590650225735 -0.7653668647301796, 1.6629392246050905 -1.1111404660392044, 1.4142135623730951 -1.414213562373095, 1.1111404660392046 -1.6629392246050905, 0.7653668647301797 -1.8477590650225735, 0.3901806440322567 -1.9615705608064609, 0. -2., -0.3901806440322564 -1.9615705608064609, -0.7653668647301795 -1.8477590650225735, -1.111140466039204 -1.6629392246050907, -1.414213562373095 -1.4142135623730951, -1.6629392246050907 -1.1111404660392044, -1.8477590650225735 -0.7653668647301798, -1.9615705608064609 -0.3901806440322572, -2. 0., -1.9615705608064609 0.3901806440322567, -1.8477590650225735 0.7653668647301792, -1.662939224605091 1.111140466039204, -1.4142135623730954 1.414213562373095, -1.1111404660392044 1.6629392246050905, -0.7653668647301807 1.847759065022573, -0.3901806440322573 1.9615705608064609, 0. 2., 0.3901806440322566 1.9615705608064609, 0.76536686473018 1.8477590650225733, 1.1111404660392037 1.662939224605091, 1.4142135623730947 1.4142135623730954, 1.6629392246050905 1.1111404660392044, 1.847759065022573 0.7653668647301808, 1.9615705608064606 0.3901806440322574, 2. 0.))
            )
        };
        check_buffer_test_case(&actual.into(), &expected_output_from_jts.into()).unwrap();
    }

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    fn jts_tests() {
        init_logging();
        assert_jts_tests_succeed("*Buffer*.xml");
    }
}
