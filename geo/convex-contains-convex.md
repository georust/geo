For Rectangle Contains Triangle, 
if triangle is not degenerate and all three triangle corners intersect the rectangle, then the rectangle contains the triangle.
three corners intersecting means that there must be some edge crossing the rectangle, satisfying the within reqirement.  

For Triangle Contains Rectangle, 
if triangle is not degenerate and all four corners intersect the rectangle, then the triangle contains the rectangle.
four corners intersecting means that there must be some edge crossing the triangle, satisfying the within reqirement.

```
rect within triangle (Contains Trait)
                        time:   [32.047 ns 32.095 ns 32.148 ns]
                        change: [-99.132% -99.127% -99.122%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

rect within triangle (Relate Trait)
                        time:   [3.3069 µs 3.3172 µs 3.3292 µs]
                        change: [-3.6471% -2.8156% -1.7563%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
```

```
triangle within rect (Contains Trait)
                        time:   [3.0131 ns 3.0317 ns 3.0511 ns]
                        change: [-99.922% -99.921% -99.921%] (p = 0.00 < 0.05)
                        Performance has improved.

triangle within rect (Relate Trait)
                        time:   [3.7826 µs 3.7919 µs 3.8013 µs]
                        change: [-1.5469% -1.1346% -0.7272%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
```

```
rect disjoint triangle (Contains Trait)
                        time:   [2.5148 ns 2.5217 ns 2.5276 ns]
                        change: [-78.872% -78.789% -78.709%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild

rect disjoint triangle (Relate Trait)
                        time:   [10.400 ns 10.423 ns 10.450 ns]
                        change: [-12.136% -11.714% -11.319%] (p = 0.00 < 0.05)
                        Performance has improved.
```

```
triangle disjoint rect (Contains Trait)
                        time:   [9.1853 ns 9.1987 ns 9.2132 ns]
                        change: [-27.199% -26.980% -26.746%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild

triangle disjoint rect (Relate Trait)
                        time:   [10.644 ns 10.676 ns 10.707 ns]
                        change: [-14.999% -14.695% -14.349%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
```