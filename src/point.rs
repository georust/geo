use std::num::Float;

#[derive(Clone)]
struct Point {
    lng: f64,
    lat: f64,
}

impl Point {
    pub fn distance(self, point: Point) -> f64 {
        let lng_sqrd = (point.lng - self.lng).powi(2);
        let lat_sqrd = (point.lat - self.lat).powi(2);
        (lng_sqrd + lat_sqrd).sqrt()
    }
}

#[test]
fn test_distance() {
    let point1 = Point {lng: 9., lat: 3.};
    let point2 = Point {lng: 6., lat: -1.};
    let distance = 5.;
    assert_eq!(distance, point1.clone().distance(point2.clone()));
    assert_eq!(distance, point2.clone().distance(point1.clone()));
}
