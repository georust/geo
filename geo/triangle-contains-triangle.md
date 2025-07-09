for two non degenerate convex polygons, if all points of a lie in b, then b contains a

   // bounding box check
        // bounding box self !contains bounding box rhs iff 
        // (1) bounding box b is degenerate and lies on bounding box a or (2) some part of b not in a  
        // if case (1), then there canno be any part of b within a
        // if case (2), then a cannot contains b
        // and rect contains rect is cheap
```
triangle contains triangle (Contains Trait)
                        time:   [16.831 ns 16.899 ns 16.968 ns]
                        change: [-99.548% -99.544% -99.540%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

triangle contains triangle (Relate Trait)
                        time:   [3.7257 µs 3.7362 µs 3.7473 µs]
                        change: [+3.1596% +3.5874% +4.0676%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
```

```
triangle disjoint triangle disjoint bounding box(Contains Trait)
                        time:   [3.4483 ns 3.4639 ns 3.4848 ns]
                        change: [-57.537% -57.185% -56.744%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

triangle disjoint triangle  disjoint bounding box (Relate Trait)
                        time:   [12.894 ns 12.927 ns 12.966 ns]
                        change: [+53.795% +54.311% +54.854%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
```

```
triangle disjoint triangle overlapping bounding box (Contains Trait)
                        time:   [12.639 ns 12.684 ns 12.730 ns]
                        change: [-99.604% -99.603% -99.601%] (p = 0.00 < 0.05)
                        Performance has improved.

triangle disjoint triangle overlapping bounding box (Relate Trait)
                        time:   [3.2598 µs 3.2698 µs 3.2807 µs]
                        change: [+2.4239% +2.8904% +3.3359%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
```
