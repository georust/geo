use crate::utils::{partial_max, partial_min};
use crate::{coord, geometry::*, CoordNum, GeometryCow};
use geo_traits::to_geo::{ToGeoCoord, ToGeoRect};
use geo_traits_ext::*;
use geo_types::private_utils::get_bounding_rect;

/// Calculation of the bounding rectangle of a geometry.
pub trait BoundingRect<T: CoordNum> {
    type Output: Into<Option<Rect<T>>>;

    /// Return the bounding rectangle of a geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::BoundingRect;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 118.34),
    /// ];
    ///
    /// let bounding_rect = line_string.bounding_rect().unwrap();
    ///
    /// assert_eq!(40.02f64, bounding_rect.min().x);
    /// assert_eq!(42.02f64, bounding_rect.max().x);
    /// assert_eq!(116.34, bounding_rect.min().y);
    /// assert_eq!(118.34, bounding_rect.max().y);
    /// ```
    fn bounding_rect(&self) -> Self::Output;
}

impl<T, G> BoundingRect<T> for G
where
    T: CoordNum,
    G: GeoTraitExtWithTypeTag + BoundingRectTrait<T, G::Tag>,
{
    type Output = G::Output;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect_trait()
    }
}

pub trait BoundingRectTrait<T, GT: GeoTypeTag>
where
    T: CoordNum,
{
    type Output: Into<Option<Rect<T>>>;

    fn bounding_rect_trait(&self) -> Self::Output;
}

impl<T, C: CoordTraitExt<T = T>> BoundingRectTrait<T, CoordTag> for C
where
    T: CoordNum,
{
    type Output = Rect<T>;

    /// Return the bounding rectangle for a `Coord`. It will have zero width
    /// and zero height.
    fn bounding_rect_trait(&self) -> Self::Output {
        Rect::new(self.to_coord(), self.to_coord())
    }
}

impl<T, P: PointTraitExt<T = T>> BoundingRectTrait<T, PointTag> for P
where
    T: CoordNum,
{
    type Output = Rect<T>;

    /// Return the bounding rectangle for a `Point`. It will have zero width
    /// and zero height.
    fn bounding_rect_trait(&self) -> Self::Output {
        match self.coord() {
            Some(coord) => Rect::new(coord.to_coord(), coord.to_coord()),
            None => {
                let zero = Coord::<T>::zero();
                Rect::new(zero, zero)
            }
        }
    }
}

impl<T, MP: MultiPointTraitExt<T = T>> BoundingRectTrait<T, MultiPointTag> for MP
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiPoint
    fn bounding_rect_trait(&self) -> Self::Output {
        get_bounding_rect(self.coord_iter())
    }
}

impl<T, L: LineTraitExt<T = T>> BoundingRectTrait<T, LineTag> for L
where
    T: CoordNum,
{
    type Output = Rect<T>;

    fn bounding_rect_trait(&self) -> Self::Output {
        Rect::new(self.start().to_coord(), self.end().to_coord())
    }
}

impl<T, LS: LineStringTraitExt<T = T>> BoundingRectTrait<T, LineStringTag> for LS
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a LineString
    fn bounding_rect_trait(&self) -> Self::Output {
        get_bounding_rect(self.coord_iter())
    }
}

impl<T, MLS: MultiLineStringTraitExt<T = T>> BoundingRectTrait<T, MultiLineStringTag> for MLS
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiLineString
    fn bounding_rect_trait(&self) -> Self::Output {
        get_bounding_rect(self.coord_iter())
    }
}

impl<T, P: PolygonTraitExt<T = T>> BoundingRectTrait<T, PolygonTag> for P
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a Polygon
    fn bounding_rect_trait(&self) -> Self::Output {
        // let line = self.exterior();
        // // get_bounding_rect(&line.0)
        // let coords = line.coords_iter().map(|c| c.to_coord());
        // get_bounding_rect(coords)
        let exterior = self.exterior_ext();
        exterior.and_then(|e| get_bounding_rect(e.coord_iter()))
    }
}

impl<T, MP: MultiPolygonTraitExt<T = T>> BoundingRectTrait<T, MultiPolygonTag> for MP
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiPolygon
    fn bounding_rect_trait(&self) -> Self::Output {
        self.polygons_ext().fold(None, |acc, p| {
            let rect = p
                .exterior_ext()
                .and_then(|e| get_bounding_rect(e.coord_iter()));
            match (acc, rect) {
                (None, None) => None,
                (Some(r), None) | (None, Some(r)) => Some(r),
                (Some(r1), Some(r2)) => Some(bounding_rect_merge(r1, r2)),
            }
        })
    }
}

impl<T: CoordNum, TT: TriangleTraitExt<T = T>> BoundingRectTrait<T, TriangleTag> for TT
where
    T: CoordNum,
{
    type Output = Rect<T>;

    fn bounding_rect_trait(&self) -> Self::Output {
        get_bounding_rect(self.coord_iter()).unwrap()
    }
}

impl<T, R: RectTraitExt<T = T>> BoundingRectTrait<T, RectTag> for R
where
    T: CoordNum,
{
    type Output = Rect<T>;

    fn bounding_rect_trait(&self) -> Self::Output {
        self.to_rect()
    }
}

impl<T, G: GeometryTraitExt<T = T>> BoundingRectTrait<T, GeometryTag> for G
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    crate::geometry_trait_ext_delegate_impl! {
       fn bounding_rect_trait(&self) -> Self::Output;
    }
}

impl<T, GC: GeometryCollectionTraitExt<T = T>> BoundingRectTrait<T, GeometryCollectionTag> for GC
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    fn bounding_rect_trait(&self) -> Self::Output {
        self.geometries_ext().fold(None, |acc, next| {
            let next_bounding_rect = next.bounding_rect_trait();

            match (acc, next_bounding_rect) {
                (None, None) => None,
                (Some(r), None) | (None, Some(r)) => Some(r),
                (Some(r1), Some(r2)) => Some(bounding_rect_merge(r1, r2)),
            }
        })
    }
}

impl<T> BoundingRect<T> for GeometryCow<'_, T>
where
    T: CoordNum,
{
    type Output = Option<Rect<T>>;

    crate::geometry_cow_delegate_impl! {
       fn bounding_rect(&self) -> Self::Output;
    }
}

// Return a new rectangle that encompasses the provided rectangles
fn bounding_rect_merge<T: CoordNum>(a: Rect<T>, b: Rect<T>) -> Rect<T> {
    Rect::new(
        coord! {
            x: partial_min(a.min().x, b.min().x),
            y: partial_min(a.min().y, b.min().y),
        },
        coord! {
            x: partial_max(a.max().x, b.max().x),
            y: partial_max(a.max().y, b.max().y),
        },
    )
}

#[cfg(test)]
mod test {
    use super::bounding_rect_merge;
    use crate::line_string;
    use crate::BoundingRect;
    use crate::{
        coord, point, polygon, Geometry, GeometryCollection, Line, LineString, MultiLineString,
        MultiPoint, MultiPolygon, Polygon, Rect,
    };

    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = line_string![];
        let bounding_rect = linestring.bounding_rect();
        assert!(bounding_rect.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 40.02f64, y: 116.34)];
        let bounding_rect = Rect::new(
            coord! {
                x: 40.02f64,
                y: 116.34,
            },
            coord! {
                x: 40.02,
                y: 116.34,
            },
        );
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 2., y: -2.),
            (x: -3., y: -3.),
            (x: -4., y: 4.)
        ];
        let bounding_rect = Rect::new(coord! { x: -4., y: -3. }, coord! { x: 2., y: 4. });
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let multiline = MultiLineString::new(vec![
            line_string![(x: 1., y: 1.), (x: -40., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 50., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 1., y: -60.)],
            line_string![(x: 1., y: 1.), (x: 1., y: 70.)],
        ]);
        let bounding_rect = Rect::new(coord! { x: -40., y: -60. }, coord! { x: 50., y: 70. });
        assert_eq!(bounding_rect, multiline.bounding_rect().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let multipoint = MultiPoint::from(vec![(1., 1.), (2., -2.), (-3., -3.), (-4., 4.)]);
        let bounding_rect = Rect::new(coord! { x: -4., y: -3. }, coord! { x: 2., y: 4. });
        assert_eq!(bounding_rect, multipoint.bounding_rect().unwrap());
    }
    #[test]
    fn polygon_test() {
        let linestring = line_string![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.),
        ];
        let line_bounding_rect = linestring.bounding_rect().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bounding_rect, poly.bounding_rect().unwrap());
    }
    #[test]
    fn multipolygon_test() {
        let mpoly = MultiPolygon::new(vec![
            polygon![(x: 0., y: 0.), (x: 50., y: 0.), (x: 0., y: -70.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 80.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: -60., y: 0.), (x: 0., y: 6.), (x: 0., y: 0.)],
        ]);
        let bounding_rect = Rect::new(coord! { x: -60., y: -70. }, coord! { x: 50., y: 80. });
        assert_eq!(bounding_rect, mpoly.bounding_rect().unwrap());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. });
        let line2 = Line::new(coord! { x: 2., y: 3. }, coord! { x: 0., y: 1. });
        assert_eq!(
            line1.bounding_rect(),
            Rect::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. },)
        );
        assert_eq!(
            line2.bounding_rect(),
            Rect::new(coord! { x: 0., y: 1. }, coord! { x: 2., y: 3. },)
        );
    }

    #[test]
    fn bounding_rect_merge_test() {
        assert_eq!(
            bounding_rect_merge(
                Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
                Rect::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. }),
            ),
            Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. }),
        );
    }

    #[test]
    fn point_bounding_rect_test() {
        assert_eq!(
            Rect::new(coord! { x: 1., y: 2. }, coord! { x: 1., y: 2. }),
            point! { x: 1., y: 2. }.bounding_rect(),
        );
    }

    #[test]
    fn geometry_collection_bounding_rect_test() {
        assert_eq!(
            Some(Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 2. })),
            GeometryCollection::new_from(vec![
                Geometry::Point(point! { x: 0., y: 0. }),
                Geometry::Point(point! { x: 1., y: 2. }),
            ])
            .bounding_rect(),
        );
    }
}
