use geo_traits::to_geo::{ToGeoCoord, ToGeoMultiLineString};
use geo_traits::*;
use geo_traits_ext::*;

use crate::Orientation::Collinear;
use crate::{CoordNum, GeoNum, GeometryCow};

/// Geometries can have 0, 1, or two dimensions. Or, in the case of an [`empty`](#is_empty)
/// geometry, a special `Empty` dimensionality.
///
/// # Examples
///
/// ```
/// use geo_types::{Point, Rect, line_string};
/// use geo::dimensions::{HasDimensions, Dimensions};
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
    /// Some geometries, like a `MultiPoint` or `GeometryCollection` may have no elements - thus no
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
    /// use geo_types::{Point, coord, LineString};
    /// use geo::HasDimensions;
    ///
    /// let line_string = LineString::new(vec![
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 10., y: 0. },
    /// ]);
    /// assert!(!line_string.is_empty());
    ///
    /// let empty_line_string: LineString = LineString::new(vec![]);
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
    /// use geo::dimensions::{Dimensions, HasDimensions};
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
    /// let geometry_collection = GeometryCollection::new_from(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::OneDimensional, geometry_collection.dimensions());
    ///
    /// let point = Point::new(10.0, 10.0);
    /// assert_eq!(Dimensions::ZeroDimensional, point.dimensions());
    ///
    /// // An `Empty` dimensionality is distinct from, and less than, being 0-dimensional
    /// let empty_collection = GeometryCollection::<f32>::new_from(vec![]);
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
    /// use geo::dimensions::{Dimensions, HasDimensions};
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
    /// let geometry_collection = GeometryCollection::new_from(vec![degenerate_line_rect.into(), degenerate_point_rect.into()]);
    /// assert_eq!(Dimensions::ZeroDimensional, geometry_collection.boundary_dimensions());
    ///
    /// let geometry_collection = GeometryCollection::<f32>::new_from(vec![]);
    /// assert_eq!(Dimensions::Empty, geometry_collection.boundary_dimensions());
    /// ```
    fn boundary_dimensions(&self) -> Dimensions;
}

impl<C: GeoNum> HasDimensions for GeometryCow<'_, C> {
    crate::geometry_cow_delegate_impl! {
        fn is_empty(&self) -> bool;
        fn dimensions(&self) -> Dimensions;
        fn boundary_dimensions(&self) -> Dimensions;
    }
}

impl<G> HasDimensions for G
where
    G: GeoTraitExtWithTypeTag + HasDimensionsTrait<G::Tag>,
{
    fn is_empty(&self) -> bool {
        self.is_empty_trait()
    }

    fn dimensions(&self) -> Dimensions {
        self.dimensions_trait()
    }

    fn boundary_dimensions(&self) -> Dimensions {
        self.boundary_dimensions_trait()
    }
}

pub trait HasDimensionsTrait<G: GeoTypeTag> {
    fn is_empty_trait(&self) -> bool;
    fn dimensions_trait(&self) -> Dimensions;
    fn boundary_dimensions_trait(&self) -> Dimensions;
}

impl<C: GeoNum, G> HasDimensionsTrait<GeometryTag> for G
where
    G: GeometryTraitExt<T = C>,
{
    crate::geometry_trait_ext_delegate_impl! {
        fn is_empty_trait(&self) -> bool;
        fn dimensions_trait(&self) -> Dimensions;
        fn boundary_dimensions_trait(&self) -> Dimensions;
    }
}

impl<C: CoordNum, P> HasDimensionsTrait<PointTag> for P
where
    P: PointTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        false
    }

    fn dimensions_trait(&self) -> Dimensions {
        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordNum, L> HasDimensionsTrait<LineTag> for L
where
    L: LineTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        false
    }

    fn dimensions_trait(&self) -> Dimensions {
        if self.start_ext().to_coord() == self.end_ext().to_coord() {
            // degenerate line is a point
            Dimensions::ZeroDimensional
        } else {
            Dimensions::OneDimensional
        }
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        if self.start_ext().to_coord() == self.end_ext().to_coord() {
            // degenerate line is a point, which has no boundary
            Dimensions::Empty
        } else {
            Dimensions::ZeroDimensional
        }
    }
}

impl<C: CoordNum, LS> HasDimensionsTrait<LineStringTag> for LS
where
    LS: LineStringTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        self.num_coords() == 0
    }

    fn dimensions_trait(&self) -> Dimensions {
        if self.num_coords() == 0 {
            return Dimensions::Empty;
        }

        // There should be at least 1 coordinate since num_coords is not 0.
        let first = unsafe { self.coord_unchecked_ext(0).to_coord() };
        if self.coords().any(|coord| first != coord.to_coord()) {
            Dimensions::OneDimensional
        } else {
            // all coords are the same - i.e. a point
            Dimensions::ZeroDimensional
        }
    }

    /// ```
    /// use geo_types::line_string;
    /// use geo::dimensions::{HasDimensions, Dimensions};
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.)];
    /// assert_eq!(Dimensions::ZeroDimensional, ls.boundary_dimensions());
    ///
    /// let ls = line_string![(x: 0.,  y: 0.), (x: 0., y: 1.), (x: 1., y: 1.), (x: 0., y: 0.)];
    /// assert_eq!(Dimensions::Empty, ls.boundary_dimensions());
    ///```
    fn boundary_dimensions_trait(&self) -> Dimensions {
        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions_trait() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<C: CoordNum, P> HasDimensionsTrait<PolygonTag> for P
where
    P: PolygonTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        self.exterior_ext()
            .map_or(true, |exterior| exterior.is_empty_trait())
    }

    fn dimensions_trait(&self) -> Dimensions {
        if let Some(exterior) = self.exterior_ext() {
            let mut coords = exterior.coord_iter();

            let Some(first) = coords.next() else {
                // No coordinates - the polygon is empty
                return Dimensions::Empty;
            };

            let Some(second) = coords.find(|next| *next != first) else {
                // All coordinates in the polygon are the same point
                return Dimensions::ZeroDimensional;
            };

            let Some(_third) = coords.find(|next| *next != first && *next != second) else {
                // There are only two distinct coordinates in the Polygon - it's collapsed to a line
                return Dimensions::OneDimensional;
            };

            Dimensions::TwoDimensional
        } else {
            Dimensions::Empty
        }
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        match self.dimensions_trait() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: CoordNum, MP> HasDimensionsTrait<MultiPointTag> for MP
where
    MP: MultiPointTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        self.num_points() == 0
    }

    fn dimensions_trait(&self) -> Dimensions {
        if self.num_points() == 0 {
            return Dimensions::Empty;
        }

        Dimensions::ZeroDimensional
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        Dimensions::Empty
    }
}

impl<C: CoordNum, MLS> HasDimensionsTrait<MultiLineStringTag> for MLS
where
    MLS: MultiLineStringTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        self.line_strings_ext().all(|ls| ls.is_empty_trait())
    }

    fn dimensions_trait(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for line in self.line_strings_ext() {
            match line.dimensions_trait() {
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

    fn boundary_dimensions_trait(&self) -> Dimensions {
        self.to_multi_line_string().is_closed();

        if self.is_closed() {
            return Dimensions::Empty;
        }

        match self.dimensions_trait() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => unreachable!("line_string cannot be 2 dimensional"),
        }
    }
}

impl<C: CoordNum, MP> HasDimensionsTrait<MultiPolygonTag> for MP
where
    MP: MultiPolygonTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        self.polygons_ext().all(|p| p.is_empty_trait())
    }

    fn dimensions_trait(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self.polygons_ext() {
            let dimensions = geom.dimensions_trait();
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        match self.dimensions_trait() {
            Dimensions::Empty | Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: GeoNum, GC> HasDimensionsTrait<GeometryCollectionTag> for GC
where
    GC: GeometryCollectionTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        if self.num_geometries() == 0 {
            true
        } else {
            self.geometries_ext().all(|g| g.is_empty_trait())
        }
    }

    fn dimensions_trait(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self.geometries_ext() {
            let dimensions = geom.dimensions_trait();
            if dimensions == Dimensions::TwoDimensional {
                // short-circuit since we know none can be larger
                return Dimensions::TwoDimensional;
            }
            max = max.max(dimensions)
        }
        max
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        let mut max = Dimensions::Empty;
        for geom in self.geometries_ext() {
            let d = geom.boundary_dimensions_trait();

            if d == Dimensions::OneDimensional {
                return Dimensions::OneDimensional;
            }

            max = max.max(d);
        }
        max
    }
}

impl<C: CoordNum, R> HasDimensionsTrait<RectTag> for R
where
    R: RectTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        false
    }

    fn dimensions_trait(&self) -> Dimensions {
        if self.min().to_coord() == self.max().to_coord() {
            // degenerate rectangle is a point
            Dimensions::ZeroDimensional
        } else if self.min().x() == self.max().x() || self.min().y() == self.max().y() {
            // degenerate rectangle is a line
            Dimensions::OneDimensional
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        match self.dimensions_trait() {
            Dimensions::Empty => {
                unreachable!("even a degenerate rect should be at least 0-Dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

impl<C: GeoNum, T> HasDimensionsTrait<TriangleTag> for T
where
    T: TriangleTraitExt<T = C>,
{
    fn is_empty_trait(&self) -> bool {
        false
    }

    fn dimensions_trait(&self) -> Dimensions {
        use crate::Kernel;
        let [c0, c1, c2] = self.coords_ext();
        let (c0, c1, c2) = (c0.to_coord(), c1.to_coord(), c2.to_coord());
        if Collinear == C::Ker::orient2d(c0, c1, c2) {
            if c0 == c1 && c1 == c2 {
                // degenerate triangle is a point
                Dimensions::ZeroDimensional
            } else {
                // degenerate triangle is a line
                Dimensions::OneDimensional
            }
        } else {
            Dimensions::TwoDimensional
        }
    }

    fn boundary_dimensions_trait(&self) -> Dimensions {
        match self.dimensions_trait() {
            Dimensions::Empty => {
                unreachable!("even a degenerate triangle should be at least 0-dimensional")
            }
            Dimensions::ZeroDimensional => Dimensions::Empty,
            Dimensions::OneDimensional => Dimensions::ZeroDimensional,
            Dimensions::TwoDimensional => Dimensions::OneDimensional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::*;

    const ONE: Coord = crate::coord!(x: 1.0, y: 1.0);
    use crate::wkt;

    #[test]
    fn point() {
        assert_eq!(
            Dimensions::ZeroDimensional,
            wkt!(POINT(1.0 1.0)).dimensions_trait()
        );
    }

    #[test]
    fn line_string() {
        assert_eq!(
            Dimensions::OneDimensional,
            wkt!(LINESTRING(1.0 1.0,2.0 2.0,3.0 3.0)).dimensions_trait()
        );
    }

    #[test]
    fn polygon() {
        assert_eq!(
            Dimensions::TwoDimensional,
            wkt!(POLYGON((1.0 1.0,2.0 2.0,3.0 3.0,1.0 1.0))).dimensions_trait()
        );
    }

    #[test]
    fn multi_point() {
        assert_eq!(
            Dimensions::ZeroDimensional,
            wkt!(MULTIPOINT(1.0 1.0)).dimensions_trait()
        );
    }

    #[test]
    fn multi_line_string() {
        assert_eq!(
            Dimensions::OneDimensional,
            wkt!(MULTILINESTRING((1.0 1.0,2.0 2.0,3.0 3.0))).dimensions_trait()
        );
    }

    #[test]
    fn multi_polygon() {
        assert_eq!(
            Dimensions::TwoDimensional,
            wkt!(MULTIPOLYGON(((1.0 1.0,2.0 2.0,3.0 3.0,1.0 1.0)))).dimensions_trait()
        );
    }

    mod empty {
        use super::*;
        #[test]
        fn empty_line_string() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(LINESTRING EMPTY) as LineString<f64>).dimensions_trait()
            );
        }

        #[test]
        fn empty_polygon() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(POLYGON EMPTY) as Polygon<f64>).dimensions_trait()
            );
        }

        #[test]
        fn empty_multi_point() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTIPOINT EMPTY) as MultiPoint<f64>).dimensions_trait()
            );
        }

        #[test]
        fn empty_multi_line_string() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTILINESTRING EMPTY) as MultiLineString<f64>).dimensions_trait()
            );
        }

        #[test]
        fn multi_line_string_with_empty_line_string() {
            let empty_line_string = wkt!(LINESTRING EMPTY) as LineString<f64>;
            let multi_line_string = MultiLineString::new(vec![empty_line_string]);
            assert_eq!(Dimensions::Empty, multi_line_string.dimensions_trait());
        }

        #[test]
        fn empty_multi_polygon() {
            assert_eq!(
                Dimensions::Empty,
                (wkt!(MULTIPOLYGON EMPTY) as MultiPolygon<f64>).dimensions_trait()
            );
        }

        #[test]
        fn multi_polygon_with_empty_polygon() {
            let empty_polygon = (wkt!(POLYGON EMPTY) as Polygon<f64>);
            let multi_polygon = MultiPolygon::new(vec![empty_polygon]);
            assert_eq!(Dimensions::Empty, multi_polygon.dimensions_trait());
        }
    }

    mod dimensional_collapse {
        use super::*;

        #[test]
        fn line_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                Line::new(ONE, ONE).dimensions_trait()
            );
        }

        #[test]
        fn line_string_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(LINESTRING(1.0 1.0)).dimensions_trait()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(LINESTRING(1.0 1.0,1.0 1.0)).dimensions_trait()
            );
        }

        #[test]
        fn polygon_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(POLYGON((1.0 1.0))).dimensions_trait()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(POLYGON((1.0 1.0,1.0 1.0))).dimensions_trait()
            );
        }

        #[test]
        fn polygon_collapsed_to_line() {
            assert_eq!(
                Dimensions::OneDimensional,
                wkt!(POLYGON((1.0 1.0,2.0 2.0))).dimensions_trait()
            );
        }

        #[test]
        fn multi_line_string_with_line_string_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0))).dimensions_trait()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0,1.0 1.0))).dimensions_trait()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTILINESTRING((1.0 1.0),(1.0 1.0))).dimensions_trait()
            );
        }

        #[test]
        fn multi_polygon_with_polygon_collapsed_to_point() {
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0)))).dimensions_trait()
            );
            assert_eq!(
                Dimensions::ZeroDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0,1.0 1.0)))).dimensions_trait()
            );
        }

        #[test]
        fn multi_polygon_with_polygon_collapsed_to_line() {
            assert_eq!(
                Dimensions::OneDimensional,
                wkt!(MULTIPOLYGON(((1.0 1.0,2.0 2.0)))).dimensions_trait()
            );
        }
    }
}
