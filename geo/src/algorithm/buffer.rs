//! Create a new geometry whose boundary is offset the specified distance from the input.

use crate::algorithm::orient::{Direction, Orient};
use crate::bool_ops::i_overlay_integration::{
    convert::{line_string_to_shape_path, multi_polygon_from_shapes, ring_to_shape_path},
    BoolOpsCoord,
};
use crate::bool_ops::{unary_union, BoolOpsNum, BooleanOps};
use crate::coord;
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
