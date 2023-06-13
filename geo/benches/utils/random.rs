#![allow(unused)]
use std::f64::consts::PI;

use geo::algorithm::{ConcaveHull, ConvexHull, MapCoords, Rotate};
use geo::geometry::*;

use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal, Standard};

// TODO: @rmanoka wonders if: it would be nice to provide these
// random-geom-samplers as `Distribution::sample` implementations directly. But
// that needs to happen in geo-types. May be related to our discussion on
// handling private-utils in geo-types, but this could be folded into a
// feature-flag too.

#[inline]
pub fn uniform_point<R: Rng>(rng: &mut R, bounds: Rect<f64>) -> Coord<f64> {
    let coords: [f64; 2] = rng.sample(Standard);
    let dims = bounds.max() - bounds.min();
    Coord {
        x: bounds.min().x + dims.x * coords[0],
        y: bounds.min().y + dims.y * coords[1],
    }
}

#[inline]
pub fn uniform_line<R: Rng>(rng: &mut R, bounds: Rect<f64>) -> Line<f64> {
    Line::new(uniform_point(rng, bounds), uniform_point(rng, bounds))
}

#[inline]
pub fn uniform_line_with_length<R: Rng>(rng: &mut R, bounds: Rect<f64>, length: f64) -> Line<f64> {
    let start = uniform_point(rng, bounds);
    let line = Line::new(start, start + (length, 0.).into());
    let angle = rng.sample::<f64, _>(Standard) * 2. * PI;
    line.rotate_around_point(angle, start.into())
}

pub fn scaled_generator(dims: Coord<f64>, scale: usize) -> impl Fn() -> Line<f64> {
    let scaling: f64 = (1 << scale) as f64;
    let bounds = Rect::new([0., 0.].into(), dims / scaling);
    let shift_bounds = Rect::new([0., 0.].into(), dims - (dims / scaling));

    move || {
        let shift = uniform_point(&mut thread_rng(), shift_bounds);
        uniform_line(&mut thread_rng(), bounds).map_coords(|mut c| {
            c.x += shift.x;
            c.y += shift.y;
            c
        })
    }
}

pub fn circular_polygon<R: Rng>(mut rng: R, steps: usize) -> Polygon<f64> {
    let mut ring = Vec::with_capacity(steps);
    let ang_step = 2. * PI / steps as f64;
    // let ang_nudge = ang_step / 100.;

    let sn = Normal::<f64>::new(0.0, 1.0).unwrap();
    let mut angle: f64 = 0.0;
    (0..steps).for_each(|_| {
        let r: f64 = sn.sample(&mut rng).abs() + 0.1;

        // let ang_nudge = sn.sample(&mut rng) * ang_nudge;
        // angle += ang_nudge;

        let (sin, cos) = angle.sin_cos();
        ring.push((r * cos, r * sin).into());

        angle += ang_step;
    });

    Polygon::new(LineString(ring), vec![])
}

pub fn steppy_polygon<R: Rng>(mut rng: R, steps: usize) -> Polygon<f64> {
    let mut ring = Vec::with_capacity(2 * steps);

    let y_step = 1.0;
    let nudge_std = y_step / 1000.0;
    let mut y = 0.0;
    let normal = Normal::new(0.0, nudge_std * nudge_std).unwrap();
    let x_shift = 50.0;

    ring.push((0.0, 0.0).into());
    (0..steps).for_each(|_| {
        let x: f64 = rng.sample::<f64, _>(Standard) * x_shift / 2.;
        y += y_step;
        ring.push((x, y).into());
    });
    ring.push((5. * x_shift, y).into());
    (0..steps).for_each(|_| {
        let x: f64 = rng.sample::<f64, _>(Standard) * x_shift;
        y -= y_step;
        // y += normal.sample(&mut rng);
        ring.push((x_shift + x, y).into());
    });

    Polygon::new(LineString(ring), vec![])
}

pub struct Samples<T>(Vec<T>);
impl<T> Samples<T> {
    pub fn sampler<'a>(&'a self) -> impl FnMut() -> &'a T {
        let mut curr = 0;
        move || {
            let ret = curr;
            curr += 1;
            curr %= self.0.len();
            &self.0[ret]
        }
    }
    pub fn from_fn<F: FnMut() -> T>(size: usize, mut proc: F) -> Self {
        Self((0..size).map(|_| proc()).collect())
    }
}
