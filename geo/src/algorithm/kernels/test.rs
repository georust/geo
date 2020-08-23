use super::*;
use num_traits::*;

// Define a SimpleFloat to test non-robustness of naive
// implementation.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct SimpleFloat(pub f64);

use std::ops::*;
use crate::has_kernel;

macro_rules! impl_ops {
	  ($t:ident, $m:ident) => {
		    impl $t for SimpleFloat {
            type Output = Self;
            fn $m(self, rhs: Self) -> Self {
                SimpleFloat((self.0).$m(rhs.0))
            }
        }
	  };
}

impl_ops!(Rem, rem);
impl_ops!(Div, div);
impl_ops!(Mul, mul);
impl_ops!(Sub, sub);
impl_ops!(Add, add);

impl One for SimpleFloat {
    fn one() -> Self {
        SimpleFloat(One::one())
    }
}
impl Zero for SimpleFloat {
    fn zero() -> Self {
        SimpleFloat(Zero::zero())
    }

    fn is_zero(&self) -> bool {
        Zero::is_zero(&self.0)
    }
}


impl Num for SimpleFloat {
    type FromStrRadixErr = ParseFloatError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Num::from_str_radix(str, radix).map(SimpleFloat)
    }
}

macro_rules! tp_method {
	  ($t:ident, $m:ident) => {
        fn $m(&self) -> Option<$t> {
            Some(self.0 as $t)
        }
	  };
}

impl ToPrimitive for SimpleFloat{
    tp_method!(i64, to_i64);
    tp_method!(u64, to_u64);
    tp_method!(f64, to_f64);
}

impl NumCast for SimpleFloat {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        NumCast::from(n).map(SimpleFloat)
    }
}

impl From<f64> for SimpleFloat {
    fn from(f: f64) -> Self {
        SimpleFloat(f)
    }
}

has_kernel!(SimpleFloat, SimpleKernel);

fn orient2d_tests<T: From<f64> + CoordinateType + HasKernel>(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    x3: f64, y3: f64,
    width: usize, height: usize,
) -> Vec<Option<WindingOrder>> {
    let p1 = Coordinate{
        x: <T as From<_>>::from(x1),
        y: <T as From<_>>::from(y1),
    };
    let p3 = Coordinate{
        x: <T as From<_>>::from(x3),
        y: <T as From<_>>::from(y3),
    };

    use float_extras::f64::nextafter;
    let mut yd2 = y2;
    let mut data = Vec::with_capacity(width * height);

    for _ in 0..height {
        let mut xd2 = x2;
        for _ in 0..width {
            let p2 = Coordinate{
                x: <T as From<_>>::from(xd2),
                y: <T as From<_>>::from(yd2),
            };
            xd2 = nextafter(xd2, std::f64::INFINITY);
            data.push(<T as HasKernel>::Ker::orient2d(p1, p2, p3));
        }
        yd2 = nextafter(yd2, std::f64::INFINITY);
    }

    data
}

use std::path::Path;
fn write_png(
    data: &[Option<WindingOrder>],
    path: &Path,
    width: usize, height: usize,
) {
    assert_eq!(data.len(), width * height);

    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    let data = data.iter().map(|w| {
        match w {
            Some(WindingOrder::Clockwise) => 0u8,
            None => 127,
            Some(WindingOrder::CounterClockwise) => 255,
        }
    }).collect::<Vec<_>>();
    writer.write_image_data(&data).unwrap();
}

#[test]
#[ignore]
fn test_naive() {
    let data = orient2d_tests::<SimpleFloat>(
        12.0, 12.0,
        0.5, 0.5,
        24.0, 24.0,
        256, 256
    );
    write_png(&data, Path::new("naive-orientation-map.png"), 256, 256);
}

#[test]
#[ignore]
fn test_robust() {
    let data = orient2d_tests::<f64>(
        12.0, 12.0,
        0.5, 0.5,
        24.0, 24.0,
        256, 256
    );
    write_png(&data, Path::new("robust-orientation-map.png"), 256, 256);
}
