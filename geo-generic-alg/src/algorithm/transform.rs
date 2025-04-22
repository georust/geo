pub use proj::{Area, Coord, Info, Proj, ProjBuilder, ProjError, ProjInfo, Transform};

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{point, Rect};

    #[test]
    fn test_transform() {
        let mut subject = {
            let point_a = point!(x: 4760096.421921f64, y: 3744293.729449f64);
            let point_b = point!(x: 4760196.421921f64, y: 3744393.729449f64);
            Rect::new(point_a, point_b)
        };

        subject
            .transform_crs_to_crs("EPSG:2230", "EPSG:26946")
            .unwrap();
        let expected = {
            let point_a = point!(x: 1450880.2910605022, y:  1141263.0111604782);
            let point_b = point!(x: 1450910.771121464, y: 1141293.4912214363);
            Rect::new(point_a, point_b)
        };
        assert_relative_eq!(subject, expected, epsilon = 0.2);
    }
}
