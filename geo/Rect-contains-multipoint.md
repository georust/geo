
all points must intersect rectangle, at least one must be within

rect contains multipoint 100 (Contains Trait)
                        time:   [4.7941 µs 4.8059 µs 4.8187 µs]
                        change: [-99.687% -99.685% -99.684%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe

rect contains multipoint 100 (Relates Trait)
                        time:   [1.5162 ms 1.5248 ms 1.5339 ms]
                        change: [-0.4346% +0.1214% +0.6936%] (p = 0.66 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

rect contains multipoint 1k (Contains Trait)
                        time:   [483.31 µs 483.95 µs 484.64 µs]
                        change: [-99.764% -99.762% -99.760%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  9 (9.00%) high severe

rect contains multipoint 1k (Relates Trait)
                        time:   [206.84 ms 207.71 ms 208.62 ms]
                        change: [-1.3311% -0.7899% -0.1755%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

rect not contains multipoint (Contains Trait)
                        time:   [487.76 µs 489.14 µs 490.75 µs]
                        change: [-99.761% -99.758% -99.755%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

rect not contains multipoint (Relates Trait)
                        time:   [209.42 ms 210.45 ms 211.53 ms]
                        change: [+1.4876% +1.9958% +2.5719%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

