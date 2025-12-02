/// Checks if `rhs` is completely contained within the interior of `self`.
///
/// More formally, the interior of `rhs` has non-empty
/// (set-theoretic) intersection but neither the interior,
/// nor the boundary of `rhs` intersects the boundary nor exterior of
/// `self`. In other words, the [DE-9IM] intersection matrix
/// of `(rhs, self)` is `T**FF*FF*`.
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # Examples
///
/// ```
/// use geo::ContainsProperly;
/// use geo::{line_string, point, Polygon};
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 2., y: 0.),
///     (x: 2., y: 2.),
///     (x: 0., y: 2.),
///     (x: 0., y: 0.),
/// ];
///
/// let polygon = Polygon::new(line_string.clone(), vec![]);
///
/// // Point in Point
/// assert!(point!(x: 2., y: 0.).contains_properly(&point!(x: 2., y: 0.)));
///
/// // Point in Linestring
/// assert!(line_string.contains_properly(&point!(x: 2., y: 0.)));
///
/// // Point in Polygon
/// assert!(!polygon.contains_properly(&point!(x: 2., y: 0.)));
/// assert!(polygon.contains_properly(&point!(x: 1., y: 1.)));
/// ```
///
/// # Performance Note
///
/// Much of this trait is currently implemented by delegating to the [`Relate`](crate::algorithm::Relate) trait - see
/// [`IntersectionMatrix::is_contains_properly`](crate::algorithm::relate::IntersectionMatrix::is_contains_properly);
/// `ContainsProperly` is faster when checking between `Polygon` and `MultiPolygon` when inputs are smaller than about 650 vertices, otherwise use [`Relate::relate().is_contains_properly`](crate::algorithm::relate::IntersectionMatrix::is_contains_properly).
pub trait ContainsProperly<Rhs = Self> {
    fn contains_properly(&self, rhs: &Rhs) -> bool;
}

mod coordinate;
mod geometry;
mod geometry_collection;
mod line;
mod line_string;
mod point;
mod polygon;
mod rect;
mod triangle;

macro_rules! impl_contains_properly_from_relate {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T> ContainsProperly<$target> for $for
            where
                T: GeoFloat
            {
                fn contains_properly(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    self.relate(target).is_contains_properly()
                }
            }
        )*
    };
}
use impl_contains_properly_from_relate;

macro_rules! impl_contains_properly_geometry_for {
    ($geom_type: ty) => {
        impl<T> ContainsProperly<Geometry<T>> for $geom_type
        where
            T: GeoFloat,
        {
            fn contains_properly(&self, geometry: &Geometry<T>) -> bool {
                match geometry {
                    Geometry::Point(g) => self.contains_properly(g),
                    Geometry::Line(g) => self.contains_properly(g),
                    Geometry::LineString(g) => self.contains_properly(g),
                    Geometry::Polygon(g) => self.contains_properly(g),
                    Geometry::MultiPoint(g) => self.contains_properly(g),
                    Geometry::MultiLineString(g) => self.contains_properly(g),
                    Geometry::MultiPolygon(g) => self.contains_properly(g),
                    Geometry::GeometryCollection(g) => self.contains_properly(g),
                    Geometry::Rect(g) => self.contains_properly(g),
                    Geometry::Triangle(g) => self.contains_properly(g),
                }
            }
        }
    };
}
use impl_contains_properly_geometry_for;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn exhaustive_compile_test() {
        use geo_types::*;
        // let c = Coord { x: 0., y: 0. };
        let pt: Point = Point::new(0., 0.);
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let ln: Line = Line::new((0., 0.), (1., 1.));

        let poly = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
        let rect = Rect::new(coord! { x: 10., y: 20. }, coord! { x: 30., y: 10. });
        let tri = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 10., y: 20. },
            coord! { x: 20., y: -10. },
        );
        let geom = Geometry::Point(pt);
        let gc = GeometryCollection::new_from(vec![geom.clone()]);
        let multi_point = MultiPoint::new(vec![pt]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        // Coordinate implementation is TODO
        //
        // let _ = c.contains_properly(&c);
        // let _ = c.contains_properly(&pt);
        // let _ = c.contains_properly(&ln);
        // let _ = c.contains_properly(&ls);
        // let _ = c.contains_properly(&poly);
        // let _ = c.contains_properly(&rect);
        // let _ = c.contains_properly(&tri);
        // let _ = c.contains_properly(&geom);
        // let _ = c.contains_properly(&gc);
        // let _ = c.contains_properly(&multi_point);
        // let _ = c.contains_properly(&multi_ls);
        // let _ = c.contains_properly(&multi_poly);

        // let _ = pt.contains_properly(&c);
        let _ = pt.contains_properly(&pt);
        let _ = pt.contains_properly(&ln);
        let _ = pt.contains_properly(&ls);
        let _ = pt.contains_properly(&poly);
        let _ = pt.contains_properly(&rect);
        let _ = pt.contains_properly(&tri);
        let _ = pt.contains_properly(&geom);
        let _ = pt.contains_properly(&gc);
        let _ = pt.contains_properly(&multi_point);
        let _ = pt.contains_properly(&multi_ls);
        let _ = pt.contains_properly(&multi_poly);

        // let _ = ln.contains_properly(&c);
        let _ = ln.contains_properly(&pt);
        let _ = ln.contains_properly(&ln);
        let _ = ln.contains_properly(&ls);
        let _ = ln.contains_properly(&poly);
        let _ = ln.contains_properly(&rect);
        let _ = ln.contains_properly(&tri);
        let _ = ln.contains_properly(&geom);
        let _ = ln.contains_properly(&gc);
        let _ = ln.contains_properly(&multi_point);
        let _ = ln.contains_properly(&multi_ls);
        let _ = ln.contains_properly(&multi_poly);

        // let _ = ls.contains_properly(&c);
        let _ = ls.contains_properly(&pt);
        let _ = ls.contains_properly(&ln);
        let _ = ls.contains_properly(&ls);
        let _ = ls.contains_properly(&poly);
        let _ = ls.contains_properly(&rect);
        let _ = ls.contains_properly(&tri);
        let _ = ls.contains_properly(&geom);
        let _ = ls.contains_properly(&gc);
        let _ = ls.contains_properly(&multi_point);
        let _ = ls.contains_properly(&multi_ls);
        let _ = ls.contains_properly(&multi_poly);

        // let _ = poly.contains_properly(&c);
        let _ = poly.contains_properly(&pt);
        let _ = poly.contains_properly(&ln);
        let _ = poly.contains_properly(&ls);
        let _ = poly.contains_properly(&poly);
        let _ = poly.contains_properly(&rect);
        let _ = poly.contains_properly(&tri);
        let _ = poly.contains_properly(&geom);
        let _ = poly.contains_properly(&gc);
        let _ = poly.contains_properly(&multi_point);
        let _ = poly.contains_properly(&multi_ls);
        let _ = poly.contains_properly(&multi_poly);

        // let _ = rect.contains_properly(&c);
        let _ = rect.contains_properly(&pt);
        let _ = rect.contains_properly(&ln);
        let _ = rect.contains_properly(&ls);
        let _ = rect.contains_properly(&poly);
        let _ = rect.contains_properly(&rect);
        let _ = rect.contains_properly(&tri);
        let _ = rect.contains_properly(&geom);
        let _ = rect.contains_properly(&gc);
        let _ = rect.contains_properly(&multi_point);
        let _ = rect.contains_properly(&multi_ls);
        let _ = rect.contains_properly(&multi_poly);

        // let _ = tri.contains_properly(&c);
        let _ = tri.contains_properly(&pt);
        let _ = tri.contains_properly(&ln);
        let _ = tri.contains_properly(&ls);
        let _ = tri.contains_properly(&poly);
        let _ = tri.contains_properly(&rect);
        let _ = tri.contains_properly(&tri);
        let _ = tri.contains_properly(&geom);
        let _ = tri.contains_properly(&gc);
        let _ = tri.contains_properly(&multi_point);
        let _ = tri.contains_properly(&multi_ls);
        let _ = tri.contains_properly(&multi_poly);

        // let _ = geom.contains_properly(&c);
        let _ = geom.contains_properly(&pt);
        let _ = geom.contains_properly(&ln);
        let _ = geom.contains_properly(&ls);
        let _ = geom.contains_properly(&poly);
        let _ = geom.contains_properly(&rect);
        let _ = geom.contains_properly(&tri);
        let _ = geom.contains_properly(&geom);
        let _ = geom.contains_properly(&gc);
        let _ = geom.contains_properly(&multi_point);
        let _ = geom.contains_properly(&multi_ls);
        let _ = geom.contains_properly(&multi_poly);

        // let _ = gc.contains_properly(&c);
        let _ = gc.contains_properly(&pt);
        let _ = gc.contains_properly(&ln);
        let _ = gc.contains_properly(&ls);
        let _ = gc.contains_properly(&poly);
        let _ = gc.contains_properly(&rect);
        let _ = gc.contains_properly(&tri);
        let _ = gc.contains_properly(&geom);
        let _ = gc.contains_properly(&gc);
        let _ = gc.contains_properly(&multi_point);
        let _ = gc.contains_properly(&multi_ls);
        let _ = gc.contains_properly(&multi_poly);

        // let _ = multi_point.contains_properly(&c);
        let _ = multi_point.contains_properly(&pt);
        let _ = multi_point.contains_properly(&ln);
        let _ = multi_point.contains_properly(&ls);
        let _ = multi_point.contains_properly(&poly);
        let _ = multi_point.contains_properly(&rect);
        let _ = multi_point.contains_properly(&tri);
        let _ = multi_point.contains_properly(&geom);
        let _ = multi_point.contains_properly(&gc);
        let _ = multi_point.contains_properly(&multi_point);
        let _ = multi_point.contains_properly(&multi_ls);
        let _ = multi_point.contains_properly(&multi_poly);

        // let _ = multi_ls.contains_properly(&c);
        let _ = multi_ls.contains_properly(&pt);
        let _ = multi_ls.contains_properly(&ln);
        let _ = multi_ls.contains_properly(&ls);
        let _ = multi_ls.contains_properly(&poly);
        let _ = multi_ls.contains_properly(&rect);
        let _ = multi_ls.contains_properly(&tri);
        let _ = multi_ls.contains_properly(&geom);
        let _ = multi_ls.contains_properly(&gc);
        let _ = multi_ls.contains_properly(&multi_point);
        let _ = multi_ls.contains_properly(&multi_ls);
        let _ = multi_ls.contains_properly(&multi_poly);

        // let _ = multi_poly.contains_properly(&c);
        let _ = multi_poly.contains_properly(&pt);
        let _ = multi_poly.contains_properly(&ln);
        let _ = multi_poly.contains_properly(&ls);
        let _ = multi_poly.contains_properly(&poly);
        let _ = multi_poly.contains_properly(&rect);
        let _ = multi_poly.contains_properly(&tri);
        let _ = multi_poly.contains_properly(&geom);
        let _ = multi_poly.contains_properly(&gc);
        let _ = multi_poly.contains_properly(&multi_point);
        let _ = multi_poly.contains_properly(&multi_ls);
        let _ = multi_poly.contains_properly(&multi_poly);
    }
}
