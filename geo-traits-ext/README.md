# Geo-Traits Extended

This crate extends the `geo-traits` crate with additional traits and
implementations. The goal is to provide a set of traits that are useful for
implementing algorithms on top of the `geo-generic-alg` crate. Most of the methods are
inspired by the `geo-types` crate, but are implemented as traits on the
`geo-traits` types. Some methods returns concrete types defined in `geo-types`,
these methods are only for computing tiny, intermediate results during
algorithm execution.
