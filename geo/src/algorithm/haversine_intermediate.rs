use crate::{Point, MEAN_EARTH_RADIUS};
use num_traits::{Float, FromPrimitive};

/// Returns a new Point along a great circle route between two existing points

pub trait HaversineIntermediate<T: Float> {
    /// Returns a new Point along a great circle route between two existing points.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::algorithm::haversine_intermediate::HaversineIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::<f64>::new(10.0, 20.0);
    /// let p2 = Point::<f64>::new(125.0, 25.0);
    /// let i20 = p1.haversine_intermediate(&p2, 0.2);
    /// let i50 = p1.haversine_intermediate(&p2, 0.5);
    /// let i80 = p1.haversine_intermediate(&p2, 0.8);
    /// let i20_should = Point::new(29.8, 29.9);
    /// let i50_should = Point::new(65.8, 37.7);
    /// let i80_should = Point::new(103.5, 33.5);
    /// assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2);
    /// assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2);
    /// assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2);
    /// ```

    fn haversine_intermediate(&self, other: &Point<T>, f: T) -> Point<T>;
    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

impl<T> HaversineIntermediate<T> for Point<T>
where
    T: Float + FromPrimitive,
{
    fn haversine_intermediate(&self, other: &Point<T>, f: T) -> Point<T> {
        let params = get_params(&self, &other);
        get_point(&params, f)
    }

    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>> {
        let params = get_params(&self, &other);
        let HaversineParams { d, .. } = params;

        let total_distance = d * T::from(MEAN_EARTH_RADIUS).unwrap();

        if total_distance <= max_dist {
            if include_ends {
                return vec![*self, *other];
            } else {
                return vec![];
            }
        }

        let number_of_points = (total_distance / max_dist).ceil();
        let interval = T::one() / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends { vec![*self] } else { vec![] };

        while current_step < T::one() {
            let point = get_point(&params, current_step);
            points.push(point);
            current_step = current_step + interval;
        }

        if include_ends {
            points.push(other.clone());
        }

        points
    }
}

struct HaversineParams<T: Float + FromPrimitive> {
    d: T,
    n: T,
    o: T,
    p: T,
    q: T,
    r: T,
    s: T,
}

fn get_point<T: Float + FromPrimitive>(params: &HaversineParams<T>, f: T) -> Point<T> {
    let one = T::one();

    let HaversineParams {
        d,
        n,
        o,
        p,
        q,
        r,
        s,
    } = *params;

    let a = ((one - f) * d).sin() / d.sin();
    let b = (f * d).sin() / d.sin();

    let x = a * n + b * o;
    let y = a * p + b * q;
    let z = a * r + b * s;

    let lat = z.atan2(x.hypot(y));
    let lon = y.atan2(x);

    Point::new(lon.to_degrees(), lat.to_degrees())
}

fn get_params<T: Float + FromPrimitive>(p1: &Point<T>, p2: &Point<T>) -> HaversineParams<T> {
    let one = T::one();
    let two = one + one;

    let lat1 = p1.y().to_radians();
    let lon1 = p1.x().to_radians();
    let lat2 = p2.y().to_radians();
    let lon2 = p2.x().to_radians();

    let (lat1_sin, lat1_cos) = lat1.sin_cos();
    let (lat2_sin, lat2_cos) = lat2.sin_cos();
    let (lon1_sin, lon1_cos) = lon1.sin_cos();
    let (lon2_sin, lon2_cos) = lon2.sin_cos();

    let m = lat1_cos * lat2_cos;

    let n = lat1_cos * lon1_cos;
    let o = lat2_cos * lon2_cos;
    let p = lat1_cos * lon1_sin;
    let q = lat2_cos * lon2_sin;

    let k = (((lat1 - lat2) / two).sin().powi(2) + m * ((lon1 - lon2) / two).sin().powi(2)).sqrt();

    let d = two * k.asin();

    HaversineParams {
        d,
        n,
        o,
        p,
        q,
        r: lat1_sin,
        s: lat2_sin,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::haversine_intermediate::HaversineIntermediate;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::<f64>::new(10.0, 20.0);
        let p2 = Point::<f64>::new(15.0, 25.0);
        let i0 = p1.haversine_intermediate(&p2, 0.0);
        let i100 = p1.haversine_intermediate(&p2, 1.0);
        assert_relative_eq!(i0.x(), p1.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i0.y(), p1.y(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.x(), p2.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.y(), p2.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::<f64>::new(10.0, 20.0);
        let p2 = Point::<f64>::new(125.0, 25.0);
        let i20 = p1.haversine_intermediate(&p2, 0.2);
        let i50 = p1.haversine_intermediate(&p2, 0.5);
        let i80 = p1.haversine_intermediate(&p2, 0.8);
        let i20_should = Point::new(29.83519, 29.94841);
        let i50_should = Point::new(65.87471, 37.72201);
        let i80_should = Point::new(103.56036, 33.50518);
        assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2);
        assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2);
        assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2);
        assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2);
    }

    #[test]
    fn should_be_north_pole_test() {
        let p1 = Point::<f64>::new(0.0, 10.0);
        let p2 = Point::<f64>::new(180.0, 10.0);
        let i50 = p1.haversine_intermediate(&p2, 0.5);
        let i50_should = Point::new(90.0, 90.0);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn should_be_start_end_test() {
        let p1 = Point::<f64>::new(30.0, 40.0);
        let p2 = Point::<f64>::new(40.0, 50.0);
        let max_dist = 1500000.0; // meters
        let include_ends = true;
        let route = p1.haversine_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, p2]);
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::<f64>::new(30.0, 40.0);
        let p2 = Point::<f64>::new(40.0, 50.0);
        let max_dist = 1000000.0; // meters
        let include_ends = true;
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5);
        let route = p1.haversine_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i50, p2]);
    }

    #[test]
    fn should_add_i25_i50_i75_test() {
        let p1 = Point::<f64>::new(30.0, 40.0);
        let p2 = Point::<f64>::new(40.0, 50.0);
        let max_dist = 400000.0; // meters
        let include_ends = true;
        let i25 = p1.clone().haversine_intermediate(&p2, 0.25);
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5);
        let i75 = p1.clone().haversine_intermediate(&p2, 0.75);
        let route = p1.haversine_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i25, i50, i75, p2]);
    }
}
