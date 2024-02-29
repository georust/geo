mod impls;
mod split;

/// Trait which includes a bunch of methods for dealing with invalid polygons.
///
/// There are a few implicit assumptions that are made when dealing with geo's types. You can read
/// more about the specific assumptions of a kind of geometry in it's own documentation.
///
/// Some of geo's algorithms act in unexpected ways if those validity assumptions are not
/// respected. An example of this is `.make_ccw_winding()` on [`LineStrings`] which might fail if
/// the linestring doesn't include unique points.
pub trait Validify {
    type ValidResult;

    /// splits invalid geometries into valid ones
    fn split_into_valid(&self) -> Self::ValidResult;
}

#[cfg(test)]
mod split_tests;
