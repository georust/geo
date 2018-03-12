use num_traits::Float;
use types::Point;
use algorithm::map_coords::MapCoords;

use proj::Proj;

/// Reproject the coordinates of a `Geometry` using `rust-proj`
pub trait Reproject<T> {
    /// Projects (or inverse-projects) the coordinates using the specified
    /// source and destination projections
    fn reproject(&self, source_proj: &Proj, dest_proj: &Proj) -> Self
    where
        T: Float;
}

impl<T, G> Reproject<T> for G
where
    T: Float,
    G: MapCoords<T, T, Output = G>,
{
    fn reproject(&self, source_proj: &Proj, dest_proj: &Proj) -> Self {
        self.map_coords(&|&(x, y)| {
            let reprojected = source_proj.project(dest_proj, Point::new(x, y));
            (reprojected.x(), reprojected.y())
        })
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;
    #[test]
    fn test_reproject() {
        let wgs84_name = "+proj=longlat +datum=WGS84 +no_defs";
        let wgs84 = Proj::new(wgs84_name).unwrap();
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000 +ellps=krass +units=m +no_defs"
        ).unwrap();
        let p = Point::new(500000., 500000.);
        let rp = p.reproject(&stereo70, &wgs84);
        assert_eq!(rp, Point::new(0.436332, 0.802851));
    }
}
