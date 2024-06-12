//! This module provides rhumb-line (a.k.a. loxodrome) geometry operations.
//! The distance, destination, and bearing implementations are adapted in part
//! from their equivalents in [Turf.js](https://turfjs.org/), which in turn are
//! adapted from the Movable Type
//! [spherical geodesy tools](https://www.movable-type.co.uk/scripts/latlong.html).
//! Turf.js is copyright its authors and the geodesy tools are copyright Chris
//! Veness; both are available under an MIT license.

use crate::{point, utils::normalize_longitude, CoordFloat, Point};
use num_traits::FromPrimitive;

mod distance;
pub use distance::RhumbDistance;

mod bearing;
pub use bearing::RhumbBearing;

mod destination;
pub use destination::RhumbDestination;

mod intermediate;
pub use intermediate::RhumbIntermediate;

mod length;
pub use length::RhumbLength;

struct RhumbCalculations<T: CoordFloat + FromPrimitive> {
    from: Point<T>,
    to: Point<T>,
    phi1: T,
    delta_lambda: T,
    delta_phi: T,
    delta_psi: T,
}

impl<T: CoordFloat + FromPrimitive> RhumbCalculations<T> {
    fn new(from: &Point<T>, to: &Point<T>) -> Self {
        let ninety = T::from(90.0).unwrap();
        let pi = T::from(std::f64::consts::PI).unwrap();
        let two = T::one() + T::one();
        let four = two + two;

        let phi1 = if from.y() < -ninety {
            -ninety
        } else if from.y() > ninety {
            ninety
        } else {
            from.y()
        };
        let phi2 = if to.y() < -ninety {
            -ninety
        } else if to.y() > ninety {
            ninety
        } else {
            to.y()
        };
        let from = Point::new(from.x(), phi1);
        let to = Point::new(to.x(), phi2);
        let phi1 = phi1.to_radians();
        let phi2 = phi2.to_radians();
        let mut delta_lambda = (to.x() - from.x()).to_radians();
        // if delta_lambda is over 180° take shorter rhumb line across the anti-meridian:
        if delta_lambda > pi {
            delta_lambda = delta_lambda - (two * pi);
        }
        if delta_lambda < -pi {
            delta_lambda = delta_lambda + (two * pi);
        }

        let delta_psi = ((phi2 / two + pi / four).tan() / (phi1 / two + pi / four).tan()).ln();
        let delta_phi = phi2 - phi1;

        Self {
            from: from,
            to: to,
            phi1,
            delta_lambda,
            delta_phi,
            delta_psi,
        }
    }

    fn theta(&self) -> T {
        self.delta_lambda.atan2(self.delta_psi)
    }

    fn delta(&self) -> T {
        let threshold = T::from(10.0e-12).unwrap();
        let q = if self.delta_psi > threshold {
            self.delta_phi / self.delta_psi
        } else {
            self.phi1.cos()
        };

        (self.delta_phi * self.delta_phi + q * q * self.delta_lambda * self.delta_lambda).sqrt()
    }

    fn intermediate(&self, fraction: T) -> Point<T> {
        let delta = fraction * self.delta();
        let theta = self.theta();
        let lambda1 = self.from.x().to_radians();
        calculate_destination(delta, lambda1, self.phi1, theta).unwrap()
    }

    fn intermediate_fill(&self, max_delta: T, include_ends: bool) -> Vec<Point<T>> {
        let theta = self.theta();
        let lambda1 = self.from.x().to_radians();

        let total_delta = self.delta();

        if total_delta <= max_delta {
            return if include_ends {
                vec![self.from, self.to]
            } else {
                vec![]
            };
        }

        let number_of_points = (total_delta / max_delta).ceil();
        let interval = T::one() / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends {
            vec![self.from]
        } else {
            vec![]
        };

        while current_step < T::one() {
            let delta = total_delta * current_step;
            let point = calculate_destination(delta, lambda1, self.phi1, theta);
            points.push(point.unwrap());
            current_step = current_step + interval;
        }

        if include_ends {
            points.push(self.to);
        }

        points
    }
}

fn calculate_destination<T: CoordFloat + FromPrimitive>(
    delta: T,
    lambda1: T,
    phi1: T,
    theta: T,
) -> Option<Point<T>> {
    let pi = T::from(std::f64::consts::PI).unwrap();
    let two = T::one() + T::one();
    let four = two + two;
    let threshold = T::from(10.0e-12).unwrap();

    let delta_phi = delta * theta.cos();
    let phi2 = phi1 + delta_phi;

    // check for some daft bugger going past the pole, normalise latitude if so
    if phi2.abs() > pi / two {
        return None;
    }

    let delta_psi = ((phi2 / two + pi / four).tan() / (phi1 / two + pi / four).tan()).ln();
    // E-W course becomes ill-conditioned with 0/0
    let q = if delta_psi.abs() > threshold {
        delta_phi / delta_psi
    } else {
        phi1.cos()
    };

    let delta_lambda = (delta * theta.sin()) / q;
    let lambda2 = lambda1 + delta_lambda;

    Some(point! {
        x: normalize_longitude(lambda2.to_degrees()),
        y: phi2.to_degrees(),
    })
}
