# Polygon Separation Algorithm Implementation Summary

## Background & Approach

We started by investigating the problem of computing the minimum separation distance between two simple polygons. After evaluating several approaches:

We have settled on **Amato's decomposition method**: Chosen as most feasible - avoids complex conic sections while maintaining O(n) optimal time complexity

## Current Implementation Status

We have a **working skeleton** of Amato's algorithm that:
- Compiles and runs basic tests
- Implements the complete 9-step algorithmic structure  
- Uses simplified/placeholder implementations for complex geometric operations
- Leverages `geo` library (this library) features properly (topological equality, Euclidean distance, etc.)
- Includes comprehensive TODO annotations for each area needing improvement

## Algorithm Structure (9 Steps)

1. **Intersection Check**: `polygons_intersect()` - Currently O(n²) edge comparison <- we can use `geo`'s Intersects trait for this.
2. **Hull Computation**: `compute_hulls()` - Uses `geo::ConvexHull` (complete)
3. **Case Classification**: `classify_case()` - Basic logic implemented
4. **Polygon R Construction**: `construct_polygon_r()` - Simplified vertex ordering
5. **Shortest Path**: `shortest_path_in_polygon()` - Direct line placeholder
6. **Separator Construction**: Extension + redundant removal - Very simplified
7. **Subproblem Decomposition**: `construct_subproblems()` - Basic structure
8. **Linearly Separable Solving**: `solve_linearly_separable_subproblem()` - Brute force O(n²)
9. **Result Integration**: Complete

## Priority Order for Improvements

### High Priority (Core Algorithm Correctness)
1. **Common Supporting Lines** (`find_common_supporting_lines_simplified`)
   - Current: Uses bounding box extremes (incorrect)
   - Needed: Rotating calipers algorithm for proper tangent computation
   - Impact: Critical for non-containing case classification

2. **Signed Distance/Half-Plane Tests** (`point_between_lines`)
   - Current: Uses sum of unsigned distances with arbitrary threshold
   - Needed: Proper signed distance computation (ax + by + c = 0 form)
   - Impact: Essential for extracting facing polygon portions
   - TASK: determine whether `geo`'s existing Euclidean.distance() implementation suffices here: apparently we need "signed distance" but I suspect it will work just fine. Let's figure that out with some tests.

3. **Polygon R Construction** (`construct_polygon_r`)
   - Current: Simple vertex concatenation without proper ordering
   - Needed: Proper vertex ordering, winding direction, valid simple polygon creation
   - Impact: Invalid polygon R breaks shortest path computation
   - TASK: we have Polygon winding functionality in `geo`: https://docs.rs/geo/latest/geo/algorithm/winding_order/trait.Winding.html. This should be enough. Let's figure that out with some tests.

### Medium Priority (Algorithm Sophistication)
4. **Shortest Path in Simple Polygon** (`shortest_path_in_polygon`)
   - Current: Direct line (may not lie within polygon)
   - Needed: Triangulation + Dijkstra, or funnel algorithm
   - Impact: Affects quality of separator construction
   - TASK: we can triangulate a Polygon using https://docs.rs/geo/latest/geo/algorithm/triangulate_earcut/trait.TriangulateEarcut.html#method.earcut_triangles. We don't have a dijkstra implementation for the resulting triangulation yet, so this is an open problem.

5. **Ray Shooting/Segment Extension** (`extend_segments_to_boundary`)
   - Current: No actual extension performed
   - Needed: Ray-polygon intersection for proper boundary extension
   - Impact: Required for correct separator construction

6. **Linearly Separable Solver** (`solve_linearly_separable_subproblem`)
   - Current: Brute force O(n²) vertex-vertex distance
   - Needed: Full Algorithm 4 from Amato (visible wedges, feasible regions, totally monotone matrix)
   - Impact: Major performance and accuracy improvement
   - TASK: this is an open problem

### Lower Priority (Optimizations)
7. **Visibility Checking** (`find_visible_segment`)
   - Current: Uses closest point without visibility verification
   - Needed: Line-polygon intersection tests for true visibility
   - TASK: we have Line-Polygon intersection checks using the Intersects trait: https://docs.rs/geo/latest/geo/algorithm/intersects/trait.Intersects.html. That should be enough, let's figure it out with some tests.

8. **Intersection Detection** (`polygons_intersect`)
   - Current: O(n²) approach works but could use Bentley-Ottmann
   - Needed: Your existing sweep line implementation integration
   - TASK: we have Polygon-Polygon intersection checks using the Intersects trait: https://docs.rs/geo/latest/geo/algorithm/intersects/trait.Intersects.html. That should be enough, let's figure it out with some tests.

9. **Redundant Segment Removal** (`remove_redundant_segments`)
   - Current: Overly simplified (first and last segments only)
   - Needed: Proper implementation per Amato's specification

## Key Resources & References

- **Primary paper**: "Determining The Separation Of Simple Polygons" (1994) by Nancy M Amato: https://link.springer.com/content/pdf/10.1007/3-540-57155-8_235
- **Geometric algorithms**: Common tangents (rotating calipers), shortest path in simple polygon

## Testing Status

- Basic tests pass (non-intersecting squares, intersecting squares)
- Algorithm structure is sound
- Individual components can be tested and improved independently
- Ready for incremental enhancement while maintaining working state
- We can copy tests from Polygon-Polygon Euclidean Distance to verify our implementation
