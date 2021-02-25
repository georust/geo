# JTS Test Runner

A tool used to compare the behavior of [GeoRust/geo] to the venerable
[JTS](https://www.osgeo.org/projects/jts/).

In particular, the contents of [./resources/testxml](./resources/testxml) were copied from 
[JTS's test files](https://github.com/locationtech/jts/blob/master/modules/tests/src/test/resources/testxml).

```
# Run only the centroid tests
let mut runner = TestRunner::new().matching_filename_glob("*Centroid.xml");
runner.run().expect("test cases failed");

# Run all tests
let mut runner = TestRunner::new();
runner.run().expect("test cases failed");
```

## GeoRust is Incomplete

Not all tests are handled, in part because JTS supports a lot of things
[GeoRust/geo] doen't support (yet!). 

For some of the things which _are_ supported, [GeoRust/geo] might diverge. This
is probably a bug, and should be investigated - precisely what this test runner
is built to find!

### Parsing New Test Case Input

Parsing test case input happens in [OperationInput](./src/input.rs#L77). 

Each type of test (Centroid, ConcaveHull, etc.) has different inputs, so will
need to be handled slightly differently.

### Running New Test Cases

Evaluating [GeoRust/geo] behavior against the expectations in the test case
input happens in [TestRunner#run](./src/runner.rs#65)

[GeoRust/geo]: https://github.com/georust/geo
