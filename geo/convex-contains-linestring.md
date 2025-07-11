Extending the concepts from contains `Line` and contains `Convex Polygon` to `LineString`  
If all `Points` of a LineString intersect a convex polygon, and there exists some `Point` not on the boundary of the polygon, then the `Convex Polygon` contains the `LineString`  
If all `Points` of a LineString intersect a convex polygon, and there exists some `Line` not on the boundary of the polygon, then part of that line must cross the `Convex Polygon`'s face and the `Convex Polygon` contains the `LineString`  

benchmarks seem to indicate that relative performance gains over `Relate` trait get better as the number of points in the `LineString` increases.  

```
rect contains linestring 40 on boundary (Contains Trait)
                        time:   [73.304 ns 74.178 ns 75.137 ns]
                        change: [-98.946% -98.934% -98.923%] (p = 0.00 < 0.05)
                        Performance has improved.

rect contains linestring 40 on boundary (Relate Trait)
                        time:   [7.5039 µs 7.5469 µs 7.5914 µs]
                        change: [+3.8975% +4.3975% +4.9324%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
```

```
rect contains linestring 4000 on boundary (Contains Trait)
                        time:   [8.5126 µs 8.6127 µs 8.7135 µs]
                        change: [-99.982% -99.982% -99.982%] (p = 0.00 < 0.05)
                        Performance has improved.

rect contains linestring 4000 on boundary (Relate Trait)
                        time:   [48.053 ms 48.151 ms 48.261 ms]
                        change: [+0.6742% +0.9155% +1.1898%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
```

```
rect contains linestring disjoint in bb (Contains Trait)
                        time:   [2.0763 ns 2.0795 ns 2.0829 ns]
                        change: [-99.921% -99.921% -99.921%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

rect contains linestring disjoint in bb (Relate Trait)
                        time:   [2.6528 µs 2.6660 µs 2.6796 µs]
                        change: [-0.7489% -0.1863% +0.4131%] (p = 0.54 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
```

```
triangle contains linestring 30 on boundary (Contains Trait)
                        time:   [151.67 ns 152.06 ns 152.47 ns]
                        change: [-97.263% -97.254% -97.244%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

triangle contains linestring 30 on boundary (Relate Trait)
                        time:   [5.5585 µs 5.5719 µs 5.5864 µs]
                        change: [+0.9877% +1.2700% +1.5679%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

```
triangle contains linestring 3000 on boundary (Contains Trait)
                        time:   [15.151 µs 15.193 µs 15.239 µs]
                        change: [-99.646% -99.644% -99.643%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

triangle contains linestring 3000 on boundary (Relate Trait)
                        time:   [4.3090 ms 4.3147 ms 4.3210 ms]
                        change: [+1.1245% +1.4144% +1.6907%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

```
triangle contains linestring disjoint in bounding box (Contains Trait)
                        time:   [10.973 ns 10.990 ns 11.007 ns]
                        change: [-99.671% -99.669% -99.666%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

triangle contains linestring disjoint in bounding box (Relate Trait)
                        time:   [3.3271 µs 3.3331 µs 3.3393 µs]
                        change: [-0.9428% -0.6279% -0.3293%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
```
