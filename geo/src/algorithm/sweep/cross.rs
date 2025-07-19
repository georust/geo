use crate::GeoFloat;
use geo_types::Line;
use std::rc::Rc;
use std::sync::Arc;

/// A 1-dimensional finite line used as input to [`Intersections`](super::Intersections).
///
/// This is implemented by [`Line`], but you can implement it on your own
/// type if you'd like to associate some other data with it.
///
/// # Cloning
///
/// Note that for usage with the `Intersections` iterator, the type must
/// also impl. `Clone`. If the custom type is not cheap to clone, use
/// either a reference to the type, a [`Rc`] or an [`Arc`]. All these
/// are supported via blanket trait implementations.
pub trait Cross {
    /// Scalar used by the line coordinates.
    type Scalar: GeoFloat;

    /// The geometry associated with this type.
    fn line(&self) -> Line<Self::Scalar>;
}

impl<T: GeoFloat> Cross for Line<T> {
    type Scalar = T;

    fn line(&self) -> Line<Self::Scalar> {
        *self
    }
}

impl<C: Cross> Cross for &C {
    type Scalar = C::Scalar;

    fn line(&self) -> Line<Self::Scalar> {
        C::line(*self)
    }
}

macro_rules! blanket_impl_smart_pointer {
    ($ty:ty) => {
        impl<T: Cross> Cross for $ty {
            type Scalar = T::Scalar;

            fn line(&self) -> Line<Self::Scalar> {
                T::line(self)
            }
        }
    };
}
blanket_impl_smart_pointer!(Box<T>);
blanket_impl_smart_pointer!(Rc<T>);
blanket_impl_smart_pointer!(Arc<T>);
