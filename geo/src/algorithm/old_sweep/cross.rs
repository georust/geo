use std::{fmt::Debug, rc::Rc, sync::Arc};

use geo_types::Line;

use super::*;
use crate::GeoFloat;

/// Interface for types that can be processed to detect crossings.
///
/// This type is implemented by [`LineOrPoint`], but users may also implement
/// this on custom types to store extra information. Any type that represents an
/// ordered line-segment may implement this.
///
/// # Cloning
///
/// Note that for usage with the planar sweep iterators, the type must
/// also impl. `Clone`. If the custom type is not cheap to clone, use
/// either a reference to the type, a [`Rc`] or an [`Arc`]. All these
/// are supported via blanket trait implementations.
pub trait Cross: Sized + Debug {
    /// Scalar used the coordinates.
    type Scalar: GeoFloat;

    /// The geometry associated with this type. Use a `Line` with the
    /// `start` and `end` coordinates to represent a point.
    fn line(&self) -> LineOrPoint<Self::Scalar>;
}

impl<T: Cross> Cross for &'_ T {
    type Scalar = T::Scalar;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        T::line(*self)
    }
}

impl<T: GeoFloat> Cross for LineOrPoint<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        *self
    }
}

impl<T: GeoFloat> Cross for Line<T> {
    type Scalar = T;

    fn line(&self) -> LineOrPoint<Self::Scalar> {
        (*self).into()
    }
}

macro_rules! blanket_impl_smart_pointer {
    ($ty:ty) => {
        impl<T: Cross> Cross for $ty {
            type Scalar = T::Scalar;

            fn line(&self) -> LineOrPoint<Self::Scalar> {
                T::line(self)
            }
        }
    };
}
blanket_impl_smart_pointer!(Box<T>);
blanket_impl_smart_pointer!(Rc<T>);
blanket_impl_smart_pointer!(Arc<T>);
