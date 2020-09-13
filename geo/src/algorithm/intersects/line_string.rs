use super::Intersects;
use crate::kernels::*;
use crate::*;

impl<T> Intersects<Line<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        self.lines().any(|l| line.intersects(&l))
    }
}

impl<T> Intersects<LineString<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        if self.0.is_empty() || linestring.0.is_empty() {
            return false;
        }
        for a in self.lines() {
            for b in linestring.lines() {
                if a.intersects(&b) {
                    return true;
                }
            }
        }

        false
    }
}
