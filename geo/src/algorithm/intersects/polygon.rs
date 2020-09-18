use super::Intersects;
use crate::contains::Contains;
use crate::kernels::*;
use crate::utils::{coord_pos_relative_to_ring, CoordPos};
use crate::*;

impl<T> Intersects<Coordinate<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, p: &Coordinate<T>) -> bool {
        coord_pos_relative_to_ring(*p, &self.exterior()) != CoordPos::Outside
            && self
                .interiors()
                .iter()
                .all(|int| coord_pos_relative_to_ring(*p, int) != CoordPos::Inside)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Polygon<T>, HasKernel);

impl<T> Intersects<Point<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, p: &Point<T>) -> bool {
        self.intersects(&p.0)
    }
}
symmetric_intersects_impl!(Point<T>, Polygon<T>, HasKernel);

impl<T> Intersects<Line<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        self.exterior().intersects(line)
            || self.interiors().iter().any(|inner| inner.intersects(line))
            || self.contains(&line.start)
            || self.contains(&line.end)
    }
}
symmetric_intersects_impl!(Line<T>, Polygon<T>, HasKernel);

impl<T> Intersects<LineString<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        // line intersects inner or outer polygon edge
        if self.exterior().intersects(linestring)
            || self
                .interiors()
                .iter()
                .any(|inner| inner.intersects(linestring))
        {
            true
        } else {
            // or if it's contained in the polygon
            linestring.0.iter().any(|c| self.contains(c))
        }
    }
}
symmetric_intersects_impl!(LineString<T>, Polygon<T>, HasKernel);

impl<T> Intersects<Rect<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, rect: &Rect<T>) -> bool {
        let p = Polygon::new(
            LineString::from(vec![
                (rect.min().x, rect.min().y),
                (rect.min().x, rect.max().y),
                (rect.max().x, rect.max().y),
                (rect.max().x, rect.min().y),
                (rect.min().x, rect.min().y),
            ]),
            vec![],
        );
        self.intersects(&p)
    }
}
symmetric_intersects_impl!(Rect<T>, Polygon<T>, HasKernel);

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        // self intersects (or contains) any line in polygon
        self.intersects(polygon.exterior()) ||
            polygon.interiors().iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(self.exterior())
    }
}
