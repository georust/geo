The contents of this directory are test fixtures copied from the venerable JTS
project.

From https://raw.githubusercontent.com/locationtech/jts/b98e823/README.md

> JTS is open source software. It is dual-licensed under:
>
> Eclipse Public License 2.0
> Eclipse Distribution License 1.0 (a BSD Style License)


The files in `testxml` match JTS commit 4a4118795, with these exceptions:

- `validate/TestRelatePL.xml`, case "P/L-2: a point and a zero-length line",
  expects `equalsTopo` to be true. This value comes from GEOS.
- These files are not in the JTS tree, and are kept from an earlier import:
  `failure/bug368jts.xml`, `failure/geos-bug368-union.xml`,
  `failure/TestOverlayFailure.xml`, `robust/ExternalRobustness.xml`,
  `robust/TestRobustOverlayError.xml`, `robust/TestRobustOverlayFixed.xml`
  and `robust/TestRobustOverlayFloat.xml`.
