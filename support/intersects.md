---
style: table.css
---

## Intersects Trait

Current status of the `Intersects` trait. The row denotes
the data-type that implements, and the column is the `Rhs`.

<table class="impltrait">
    <thead>
        <tr>
            <th>Type</th>
            <!-- 0-D -->
            <th>C</th>
            <th>Pt</th>
            <th>MPt</th>
            <!-- 1-D -->
            <th>L</th>
            <th>LS</th>
            <th>MLS</th>
            <!-- 2-D -->
            <th>Tr</th>
            <th>Rc</th>
            <th>Pl</th>
            <th>MPl</th>
            <!-- Collections -->
            <th>Gm</th>
            <th>GmC</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Coordinate (C)</td>
            <!-- 0-D -->
            <td class="good">Y</td>
            <td class="good">Y</td>
            <td class="good">S</td>
            <!-- 1-D -->
            <td class="good">S</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 2-D -->
            <td class="good">S</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- Collections -->
            <td class="good">S</td>
            <td class="good">S</td>
        </tr>
        <tr>
            <td>Point (Pt)</td>
            <td colspan="12" class="good">blanket impl using Coordinate</td>
        </tr>
        <tr>
            <td>MultiPoint (MPt)</td>
            <td colspan="12" class="good">blanket impl using Point</td>
        </tr>
        <tr>
            <td>Line (L)</td>
            <!-- 0-D -->
            <td class="good">Y</td>
            <td class="good">Y</td>
            <td class="good">S</td>
            <!-- 1-D -->
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 2-D -->
            <td class="good">S</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- Collections -->
            <td class="good">S</td>
            <td class="good">S</td>
        </tr>
        <tr>
            <td>LineString (LS)</td>
            <td class="good" colspan="12">blanket impl from Line using <code>self.lines.any()</code></td>
        </tr>
        <tr>
            <td>MultiLineString (MLS)</td>
            <td class="good" colspan="12">blanket impl using LineString</td>
        </tr>
        <tr>
            <td>Triangle (Tr)</td>
            <td class="good" colspan="12">blanket impl using Polygon</td>
        </tr>
        <tr>
            <td>Rect (Rc)</td>
            <!-- 0-D -->
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 1-D -->
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 2-D -->
            <td class="good">S</td>
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- Collections -->
            <td class="good">S</td>
            <td class="good">S</td>
        </tr>
        <tr>
            <td>Polygon (Pl)</td>
            <!-- 0-D -->
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 1-D -->
            <td class="good">Y</td>
            <td class="good">S</td>
            <td class="good">S</td>
            <!-- 2-D -->
            <td class="good">S</td>
            <td class="good">Y</td>
            <td class="good">Y</td>
            <td class="good">S</td>
            <!-- Collections -->
            <td class="good">S</td>
            <td class="good">S</td>
        </tr>
        <tr>
            <td>MultiPolygon (MPl)</td>
            <td colspan="12" class="good">blanket impl using Polygon</td>
        </tr>
        <tr>
            <td>Geometry (Gm)</td>
            <td colspan="12" class="good">blanket impl using rest</td>
        </tr>
        <tr>
            <td>GeometryCollection (GmC)</td>
            <td colspan="12" class="good">blanket impl using rest</td>
        </tr>
    </tbody>
</table>

### Legend

| Symbol | Meaning                                  |
| :---:  | :---                                     |
| Y      | Implemented                              |
| S      | Implemented via symmetry                 |
| B      | Implemented with (a few) documented bugs |
| N      | Not implemented                          |
