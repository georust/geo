use std::num::Float;

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn distance(self, point: Point) -> f64 {
        let x_sqrd = (point.x - self.x).powi(2);
        let y_sqrd = (point.y - self.y).powi(2);
        (x_sqrd + y_sqrd).sqrt()
    }
}

#[test]
fn test_distance() {
    let point1 = Point {x: 9., y: 3.};
    let point2 = Point {x: 6., y: -1.};
    let distance = 5.;
    assert_eq!(distance, point1.clone().distance(point2.clone()));
    assert_eq!(distance, point2.clone().distance(point1.clone()));
}
