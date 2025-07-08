For the `Contains` predicate, the line is contained if all points of the line intersect the shape, and at least one part of the line lies does not lie on the shape's boundary.  
For a convex shape, if both ends of a line intersect the shape, the shape contains the line.  
However, if the both ends of the line line on the same segment of the boundary, then it must lie on the shape's boundary.  
It is not possible for a line to be on more than one segment of the boundary for Rectangle and Triangle.
- For both these shape, it is impossible for adjacent segments of the boundary to be coliniear.
- The possible exception is a degerate case where the line is a point or a line.
- In degenerate cases, the shape becomes a line or a point, and the line is either disjoint, or contained by the boundary.  

Therefore it is sufficient to check that both ends intersect the shape,
and that they do not intersect the same segment of the boundary.  
if any end of the line lies within the polygon(i.e not on the boundary), we can short circuit the check

```
line within rect (Contains Trait)
                        time:   [2.0043 ns 2.0093 ns 2.0152 ns]
                        change: [-99.921% -99.920% -99.920%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  3 (3.00%) high severe

line within rect (Relates Trait)
                        time:   [2.4976 µs 2.5088 µs 2.5205 µs]
                        change: [-0.2887% +0.1245% +0.5210%] (p = 0.54 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

```
line disjoint rect (Contains Trait)
                        time:   [1.9890 ns 1.9913 ns 1.9937 ns]
                        change: [-78.327% -78.253% -78.182%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe

line disjoint rect (Relates Trait)
                        time:   [8.2672 ns 8.2721 ns 8.2774 ns]
                        change: [-10.612% -10.421% -10.228%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  2 (2.00%) high severe
```

```
line along rect (Contains Trait)
                        time:   [11.960 ns 11.976 ns 11.995 ns]
                        change: [-99.580% -99.578% -99.577%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe

line along rect (Relates Trait)
                        time:   [2.8608 µs 2.8700 µs 2.8829 µs]
                        change: [-0.1797% +0.2629% +0.7349%] (p = 0.27 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
```

```
line within triangle (Contains Trait)
                        time:   [13.358 ns 13.371 ns 13.388 ns]
                        change: [-99.483% -99.481% -99.479%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  6 (6.00%) high mild
  2 (2.00%) high severe

line within triangle (Relates Trait)
                        time:   [2.5793 µs 2.5879 µs 2.5968 µs]
                        change: [-0.0517% +0.4215% +0.8637%] (p = 0.07 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
```

```
line disjoint triangle (Contains Trait)
                        time:   [5.3985 ns 5.4122 ns 5.4268 ns]
                        change: [-56.951% -56.754% -56.560%] (p = 0.00 < 0.05)
                        Performance has improved.

line disjoint triangle (Relates Trait)
                        time:   [10.561 ns 10.613 ns 10.688 ns]
                        change: [-16.871% -16.486% -16.071%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
```

```
line along triangle (Contains Trait)
                        time:   [21.239 ns 21.267 ns 21.307 ns]
                        change: [-99.263% -99.259% -99.256%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe

line along triangle (Relates Trait)
                        time:   [2.8540 µs 2.8593 µs 2.8655 µs]
                        change: [-1.2264% -0.6838% -0.1442%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
```
