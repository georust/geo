# geo-generic-tests

A crate for testing the geo-* ecosystem of crates with a focus on generics.
This is not a crate for being reused, only for simulating the usage of the
geo-traits, geo-traits-ext and geo-generic-alg crates.

## WKB Module

The WKB (Well-Known Binary) module provides functionality for working with OGC Well-Known Binary
format for geometric objects.

## Usage

Run the binary with:

```
cargo run
```

To enable logging, use:

```
RUST_LOG=debug cargo run
```

## Purpose

This binary provides a practical test environment for demonstrating and verifying 
the functionality of generic geometric algorithms and traits from the geo-* family 
of crates. 