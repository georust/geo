use num_traits::Float;
use types::Point;
use algorithm::map_coords::MapCoords;

use proj::Proj; 

pub trait Reproject<T> {
    /// Reproject the coordinates of a `Geometry` using `rust-proj`
    fn reproject(&self, sproj: &Proj, dproj: &Proj) -> Self
    where
        T: Float;
}

impl<T, G> Reproject<T> for G
where
    T: Float,
    G: MapCoords<T, T, Output = G>,
{
    fn reproject(&self, sproj: &Proj, dproj: &Proj) -> Self {
        self.map_coords(&|&(x, y)| {
            let reprojected = sproj.project(dproj, Point::new(x, y));
            (reprojected.x(), reprojected.y())
        })
    }
}
