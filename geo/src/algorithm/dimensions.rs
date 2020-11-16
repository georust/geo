use crate::{
    CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// Geometries can have 0, 1, or two dimensions. Or, in the case of an [`empty`](#is_empty)
/// geometry, a special `Empty` dimensionality.
///
/// # Examples
///
/// ```
/// use geo_types::{Point, Rect, line_string};
/// use geo::algorithm::dimensions::{HasDimensions, Dimensions};
///
/// let point = Point::new(0.0, 5.0);
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 5.0, y: 5.0), (x: 0.0, y: 5.0)];
/// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
/// assert_eq!(Dimensions::ZeroDimensional, point.dimensions());
/// assert_eq!(Dimensions::OneDimensional, line_string.dimensions());
/// assert_eq!(Dimensions::TwoDimensional, rect.dimensions());
///
/// assert!(point.dimensions() < line_string.dimensions());
/// assert!(rect.dimensions() > line_string.dimensions());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum Dimensions {
    /// Some geometries, like a `MultiPoint` or `GeometryColletion` may have no elements - thus no
    /// dimensions. Note that this is distinct from being `ZeroDimensional`, like a `Point`.
    Empty,
    /// Dimension of a point
    ZeroDimensional,
    /// Dimension of a line or curve
    OneDimensional,
    /// Dimension of a surface
    TwoDimensional,
}

/// Operate on the dimensionality of geometries.
pub trait HasDimensions {
    /// Some geometries, like a `MultiPoint`, can have zero coordinates - we call these `empty`.
    ///
    /// Types like `Point` and `Rect`, which have at least one coordinate by construction, can
    /// never be considered empty.
    /// ```
    /// use geo_types::{Point, Coordinate, LineString};
    /// use geo::algorithm::dimensions::HasDimensions;
    ///
    /// let line_string = LineString(vec![
    ///     Coordinate { x: 0., y: 0. },
    ///     Coordinate { x: 10., y: 0. },
    /// ]);
    /// assert!(!line_string.is_empty());
    ///
    /// let empty_line_string: LineString<f64> = LineString(vec![]);
    /// assert!(empty_line_string.is_empty());
    ///
    /// let point = Point::new(0.0, 0.0);
    /// assert!(!point.is_empty());
    /// ```
    fn is_empty(&self) -> bool;

    /// The dimensions of some geometries are fixed, e.g. a Point always has 0 dimensions. However
    /// for others, the dimensionality depends on the specific geometry instance - for example
    /// typical `Rect`s are 2-dimensional, but it's possible to create degenerate `Rect`s which
    /// have either 1 or 0 dimensions.
    ///
    /// ## Examples
    ///
    /// ```
    /// use geo_types::{GeometryCollection, Rect, Point};
    /// use geo::algorithm::dimensions::{Dimensions, HasDimensions};
    ///
    /// // normal rectangle
    /// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::TwoDimensional, rect.dimensions());
    ///
    /// // "rectangle" with zero height degenerates to a line
    /// let degenerate_line_rect = Rect::new((0.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::OneDimensional, degenerate_line_rect.dimensions());
    ///
    /// // "rectangle" with zero height and zero width degenerates to a point
    /// let degenerate_point_rect = Rect::new((10.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::ZeroDimensional, degenerate_point_rect.dimensions());
    ///
    /// // collections inherit the greatest dimensionality of their elements
    /// let geometry_collection = GeometryCollection(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::OneDimensional, geometry_collection.dimensions());
    ///
    /// let point = Point::new(10.0, 10.0);
    /// assert_eq!(Dimensions::ZeroDimensional, point.dimensions());
    ///
    /// // An `Empty` dimensionality is distinct from, and less than, being 0-dimensional
    /// let empty_collection = GeometryCollection::<f32>(vec![]);
    /// assert_eq!(Dimensions::Empty, empty_collection.dimensions());
    /// assert!(empty_collection.dimensions() < point.dimensions());
    /// ```
    fn dimensions(&self) -> Dimensions;

    /// The dimensions of the `Geometry`'s boundary, as used by OGC-SFA.
    ///
    /// ## Examples
    ///
    /// ```
    /// use geo_types::{GeometryCollection, Rect, Point};
    /// use geo::algorithm::dimensions::{Dimensions, HasDimensions};
    ///
    /// // a point has no boundary
    /// let point = Point::new(10.0, 10.0);
    /// assert_eq!(Dimensions::Empty, point.boundary_dimensions());
    ///
    /// // a typical rectangle has a *line* (one dimensional) boundary
    /// let rect = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::OneDimensional, rect.boundary_dimensions());
    ///
    /// // a "rectangle" with zero height degenerates to a line, whose boundary is two points
    /// let degenerate_line_rect = Rect::new((0.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::ZeroDimensional, degenerate_line_rect.boundary_dimensions());
    ///
    /// // a "rectangle" with zero height and zero width degenerates to a point,
    /// // and points have no boundary
    /// let degenerate_point_rect = Rect::new((10.0, 10.0), (10.0, 10.0));
    /// assert_eq!(Dimensions::Empty, degenerate_point_rect.boundary_dimensions());
    ///
    /// // collections inherit the greatest dimensionality of their elements
    /// let geometry_collection = GeometryCollection(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::ZeroDimensional, geometry_collection.boundary_dimensions());
    ///
    /// let geometry_collection = GeometryCollection::<f32>(vec![]);
    /// assert_eq!(Dimensions::Empty, geometry_collection.boundary_dimensions());
    /// ```
    fn boundary_dimensions(&self) -> Dimensions;
}

impl<C: CoordinateType> HasDimensions for Geometry<C> {
    fn is_empty(&self) -> bool {
        match self {
            Geometry::Point(g) => g.is_empty(),
            Geometry::Line(g) => g.is_empty(),
            Geometry::LineString(g) => g.is_empty(),
            Geometry::Polygon(g) => g.is_empty(),
            Geometry::MultiPoint(g) => g.is_empty(),
            Geometry::MultiLineString(g) => g.is_empty(),
            Geometry::MultiPolygon(g) => g.is_empty(),
            Geometry::GeometryCollection(g) => g.is_empty(),
            Geometry::Rect(g) => g.is_empty(),
            Geometry::Triangle(g) => g.is_empty(),
        }
    }

    fn dimensions(&self) -> Dimensions {
        match self {
            Geometry::Point(g) => g.dimensions(),
            Geometry::Line(g) => g.dimensions(),
            Geometry::LineString(g) => g.dimensions(),
            Geometry::Polygon(g) => g.dimensions(),
            Geometry::MultiPoint(g) => g.dimensions(),
            Geometry::MultiLineString(g) => g.dimensions(),
            Geometry::MultiPolygon(g) => g.dimensions(),
            Geometry::GeometryCollection(g) => g.dimensions(),
            Geometry::Rect(g) => g.dimensions(),
            Geometry::Triangle(g) => g.dimensions(),
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self {
            Geometry::Point(g) => g.boundary_dimensions(),
            Geometry::Line(g) => g.boundary_dimensions(),
            Geometry::LineString(g) => g.boundary_dimensions(),
            Geometry::Polygon(g) => g.boundary_dimensions(),
            Geometry::MultiPoint(g) => g.boundary_dimensions(),
            Geometry::MultiLineString(g) => g.boundary_dimensions(),
            Geometry::MultiPolygon(g) => g.boundary_dimensions(),
            Geometry::GeometryCollection(g) => g.boundary_dimensions(),
            Geometry::Rect(g) => g.boundary_dimensions(),
            Geometry::Triangle(g) => g.boundary_dimensions(),
        }
    }
}

impl<C: CoordinateType> HasDimensions for Point<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordinateType> HasDimensions for Line<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // degenerate line is a point
            Dimensions::ZeroDimensional
        } else {
            Dimensions::OneDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.start == self.end {
            // degenerate line is a point, which has no boundary
            Dimensions::Empty
        } else {
            Dimensions::ZeroDimensional
        }
    }
}

impl<C: CoordinateType> HasDimensions for LineString<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        debug_assert!(self.0.len() > 1, "invalid line_string with 1 coord");
        let first = self.0[0];
        if self.0.iter().any(|&coord| first != coord) {
            Dimensions::OneDimensional
        } else {
            // all coords are the same - i.e. a point
            Dimensions::ZeroDimensional
        }
    }

    /// ```
    /// use geo_types::line_string;
    /// use geo::algorithm::dimensions::{HasDimensions, Dimensions};
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.)];
    /// assert_eq!(Dimensions::ZeroDimensional, ls.boundary_dimensions());
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.), (x: 0., y: 0.)];
    /// assert_eq!(Dimensions::Empty, ls.boundary_dimensions());
    ///```
    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<C: CoordinateType> HasDimensions for Polygon<C> {
    fn is_empty(&self) -> bool {
        self.exterior().is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        Dimensions::TwoDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::OneDimensional
    }
}

impl<C: CoordinateType> HasDimensions for MultiPoint<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::OneDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordinateType> HasDimensions for MultiLineString<C> {
    fn is_empty(&self) -> bool {
        self.iter().all(LineString::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for line in &self.0 {
            match line.dimensions() {
                Dimensions::Empty => {}
                Dimensions::ZeroDimensional => max = Dimensions::ZeroDimensional,
                Dimensions::OneDimensional => {
                    // return early since we know multi line string dimensionality cannot exceed
                    // 1-d
                    return Dimensions::OneDimensional;
                }
                Dimensions::TwoDimensional => unreachable!("MultiLineString cannot be 2d"),
            }
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<C: CoordinateType> HasDimensions for MultiPolygon<C> {
    fn is_empty(&self) -> bool {
        self.iter().all(Polygon::is_empty)
    }

    fn dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::TwoDimensional
    }

    fn boundary_dimensions(&self) -> Dimensions {
        if self.0.is_empty() {
            return Dimensions::Empty;
        }

        Dimensions::OneDimensional
    }
}

impl<C: CoordinateType> HasDimensions for GeometryCollection<C> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.iter().all(Geometry::is_empty)
        }
    }

    fn dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let dimensions = geom.dimensions();
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self {
            let d = geom.boundary_dimensions();

            if d == Dimensions::OneDimensional {
                return Dimensions::OneDimensional;
            }

            max = max.max(d);
        }
        max
    }
}

impl<C: CoordinateType> HasDimensions for Rect<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.min() == self.max() {
            // degenerate rectangle is a point
            Dimensions::ZeroDimensional
        } else if self.min().x == self.max().x || self.min().y == self.max().y {
            // degenerate rectangle is a line
            Dimensions::OneDimensional
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("even a degenerate rect should be at least 0-Dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: CoordinateType> HasDimensions for Triangle<C> {
    fn is_empty(&self) -> bool {
        false
    }

    fn dimensions(&self) -> Dimensions {
        if self.0 == self.1 && self.1 == self.2 {
            // degenerate triangle is a point
            Dimensions::ZeroDimensional
        } else if self.0 == self.1 || self.1 == self.2 || self.2 == self.0 {
            // degenerate triangle is a line
            Dimensions::OneDimensional
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions(&self) -> Dimensions {
        match self.dimensions() {
            Dimensions::Empty => {
                unreachable!("even a degenerate triangle should be at least 0-dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}
