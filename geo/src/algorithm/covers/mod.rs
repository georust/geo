/// Checks if `rhs` is completely contained within `self`.
/// More formally, the interior of `rhs` has non-empty
/// (set-theoretic) intersection but neither the interior,
/// nor the boundary of `rhs` intersects the exterior of
/// `self`. In other words, the [DE-9IM] intersection matrix
/// of `(rhs, self)` is `T*F**F***`.
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # Examples
///
/// ```
/// use geo::Covers;
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
/// assert!(point!(x: 2., y: 0.).covers(&point!(x: 2., y: 0.)));
///
/// // Point in Linestring
/// assert!(line_string.covers(&point!(x: 2., y: 0.)));
///
/// // Point in Polygon
/// assert!(polygon.covers(&point!(x: 1., y: 1.)));
/// ```
pub trait Covers<Rhs = Self> {
    fn covers(&self, rhs: &Rhs) -> bool;
}

pub(crate) mod line_string;
pub(crate)mod point;
pub(crate)mod polygon;

pub(crate) mod line;
pub(crate) mod rect;
pub(crate) mod triangle;

pub(crate) mod geometry;
pub(crate) mod geometry_collection;

macro_rules! impl_covers_from_relate {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T> Covers<$target> for $for
            where
                T: GeoFloat
            {
                fn covers(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    self.relate(target).is_covers()
                }
            }
        )*
    };
}
pub(crate) use impl_covers_from_relate;


macro_rules! impl_covers_convex_poly {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T> Covers<$target> for $for
            where
                T: GeoNum,
                Self: Intersects<Coord<T>>,
            {
                fn covers(&self, target: &$target) -> bool {
                    target.coords_iter().all(|pt| self.intersects(&pt))
                }
            }
        )*
    };
}
pub(crate) use impl_covers_convex_poly;

macro_rules! impl_covers_geometry_for {
    ($geom_type: ty) => {
        impl<T> Covers<Geometry<T>> for $geom_type
        where
            T: GeoFloat,
        {
            fn covers(&self, geometry: &Geometry<T>) -> bool {
                match geometry {
                    Geometry::Point(g) => self.covers(g),
                    Geometry::Line(g) => self.covers(g),
                    Geometry::LineString(g) => self.covers(g),
                    Geometry::Polygon(g) => self.covers(g),
                    Geometry::MultiPoint(g) => self.covers(g),
                    Geometry::MultiLineString(g) => self.covers(g),
                    Geometry::MultiPolygon(g) => self.covers(g),
                    Geometry::GeometryCollection(g) => self.covers(g),
                    Geometry::Rect(g) => self.covers(g),
                    Geometry::Triangle(g) => self.covers(g),
                }
            }
        }
    };
}
pub(crate) use impl_covers_geometry_for;

// ┌───────┐
// │ Tests │
// └───────┘

#[cfg(test)]
mod test {
    use crate::line_string;
    use crate::Covers;
    use crate::Relate;
    use crate::*;

    #[test]
    fn exhaustive_compile_test() {
        use geo_types::{GeometryCollection, Triangle};
        let c: Coord = Coord::zero();
        let pt: Point = Point::new(0., 0.);
        let ln: Line = Line::new((0., 0.), (1., 1.));
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
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
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        let _ = c.covers(&c);
        let _ = c.covers(&pt);
        let _ = c.covers(&ln);
        let _ = c.covers(&ls);
        let _ = c.covers(&poly);
        let _ = c.covers(&rect);
        let _ = c.covers(&tri);
        let _ = c.covers(&geom);
        let _ = c.covers(&gc);
        let _ = c.covers(&multi_point);
        let _ = c.covers(&multi_ls);
        let _ = c.covers(&multi_poly);

        let _ = pt.covers(&c);
        let _ = pt.covers(&pt);
        let _ = pt.covers(&ln);
        let _ = pt.covers(&ls);
        let _ = pt.covers(&poly);
        let _ = pt.covers(&rect);
        let _ = pt.covers(&tri);
        let _ = pt.covers(&geom);
        let _ = pt.covers(&gc);
        let _ = pt.covers(&multi_point);
        let _ = pt.covers(&multi_ls);
        let _ = pt.covers(&multi_poly);

        let _ = ln.covers(&c);
        let _ = ln.covers(&pt);
        let _ = ln.covers(&ln);
        let _ = ln.covers(&ls);
        let _ = ln.covers(&poly);
        let _ = ln.covers(&rect);
        let _ = ln.covers(&tri);
        let _ = ln.covers(&geom);
        let _ = ln.covers(&gc);
        let _ = ln.covers(&multi_point);
        let _ = ln.covers(&multi_ls);
        let _ = ln.covers(&multi_poly);

        let _ = ls.covers(&c);
        let _ = ls.covers(&pt);
        let _ = ls.covers(&ln);
        let _ = ls.covers(&ls);
        let _ = ls.covers(&poly);
        let _ = ls.covers(&rect);
        let _ = ls.covers(&tri);
        let _ = ls.covers(&geom);
        let _ = ls.covers(&gc);
        let _ = ls.covers(&multi_point);
        let _ = ls.covers(&multi_ls);
        let _ = ls.covers(&multi_poly);

        let _ = poly.covers(&c);
        let _ = poly.covers(&pt);
        let _ = poly.covers(&ln);
        let _ = poly.covers(&ls);
        let _ = poly.covers(&poly);
        let _ = poly.covers(&rect);
        let _ = poly.covers(&tri);
        let _ = poly.covers(&geom);
        let _ = poly.covers(&gc);
        let _ = poly.covers(&multi_point);
        let _ = poly.covers(&multi_ls);
        let _ = poly.covers(&multi_poly);
        let _ = rect.covers(&pt);
        let _ = rect.covers(&ln);
        let _ = rect.covers(&ls);
        let _ = rect.covers(&poly);
        let _ = rect.covers(&rect);
        let _ = rect.covers(&tri);
        let _ = rect.covers(&geom);
        let _ = rect.covers(&gc);
        let _ = rect.covers(&multi_point);
        let _ = rect.covers(&multi_ls);
        let _ = rect.covers(&multi_poly);

        let _ = tri.covers(&c);
        let _ = tri.covers(&pt);
        let _ = tri.covers(&ln);
        let _ = tri.covers(&ls);
        let _ = tri.covers(&poly);
        let _ = tri.covers(&rect);
        let _ = tri.covers(&tri);
        let _ = tri.covers(&geom);
        let _ = tri.covers(&gc);
        let _ = tri.covers(&multi_point);
        let _ = tri.covers(&multi_ls);
        let _ = tri.covers(&multi_poly);

        let _ = geom.covers(&c);
        let _ = geom.covers(&pt);
        let _ = geom.covers(&ln);
        let _ = geom.covers(&ls);
        let _ = geom.covers(&poly);
        let _ = geom.covers(&rect);
        let _ = geom.covers(&tri);
        let _ = geom.covers(&geom);
        let _ = geom.covers(&gc);
        let _ = geom.covers(&multi_point);
        let _ = geom.covers(&multi_ls);
        let _ = geom.covers(&multi_poly);

        let _ = gc.covers(&c);
        let _ = gc.covers(&pt);
        let _ = gc.covers(&ln);
        let _ = gc.covers(&ls);
        let _ = gc.covers(&poly);
        let _ = gc.covers(&rect);
        let _ = gc.covers(&tri);
        let _ = gc.covers(&geom);
        let _ = gc.covers(&gc);
        let _ = gc.covers(&multi_point);
        let _ = gc.covers(&multi_ls);
        let _ = gc.covers(&multi_poly);

        let _ = multi_point.covers(&c);
        let _ = multi_point.covers(&pt);
        let _ = multi_point.covers(&ln);
        let _ = multi_point.covers(&ls);
        let _ = multi_point.covers(&poly);
        let _ = multi_point.covers(&rect);
        let _ = multi_point.covers(&tri);
        let _ = multi_point.covers(&geom);
        let _ = multi_point.covers(&gc);
        let _ = multi_point.covers(&multi_point);
        let _ = multi_point.covers(&multi_ls);
        let _ = multi_point.covers(&multi_poly);

        let _ = multi_ls.covers(&c);
        let _ = multi_ls.covers(&pt);
        let _ = multi_ls.covers(&ln);
        let _ = multi_ls.covers(&ls);
        let _ = multi_ls.covers(&poly);
        let _ = multi_ls.covers(&rect);
        let _ = multi_ls.covers(&tri);
        let _ = multi_ls.covers(&geom);
        let _ = multi_ls.covers(&gc);
        let _ = multi_ls.covers(&multi_point);
        let _ = multi_ls.covers(&multi_ls);
        let _ = multi_ls.covers(&multi_poly);

        let _ = multi_poly.covers(&c);
        let _ = multi_poly.covers(&pt);
        let _ = multi_poly.covers(&ln);
        let _ = multi_poly.covers(&ls);
        let _ = multi_poly.covers(&poly);
        let _ = multi_poly.covers(&rect);
        let _ = multi_poly.covers(&tri);
        let _ = multi_poly.covers(&geom);
        let _ = multi_poly.covers(&gc);
        let _ = multi_poly.covers(&multi_point);
        let _ = multi_poly.covers(&multi_ls);
        let _ = multi_poly.covers(&multi_poly);
    }
}
