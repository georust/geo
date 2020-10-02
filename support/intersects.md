---
style: table.css
---

## Status

Current status of the `Intersects` trait.

<table>
    <thead>
        <tr>
            <th>Type</th>
            <th>C</th>
            <th>Pt</th>
            <th>MPt</th>
            <th>L</th>
            <th>LS</th>
            <th>MLS</th>
            <th>Pl</th>
            <th>Tr</th>
            <th>Rc</th>
            <th>MPl</th>
            <th>Gm</th>
            <th>GmC</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Coord (C)</td>
            <td>Yes</td>
            <td>Sym</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
        <tr>
            <td>Point (Pt)</td>
            <td colspan="12">blanket impl using Coordinate</td>
        </tr>
        <tr>
            <td>MultiPoint (MPt)</td>
            <td colspan="12">blanket impl using Point</td>
        </tr>
        <tr><td>Line (L)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
        <tr><td>LineString (LS)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
        <tr>
            <td>MultiLineString (MLS)</td>
            <td colspan="12">blanket impl using LineString</td>
        </tr>
        <tr><td>Polygon (Pl)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
        <tr>
            <td>Triangle (Tr)</td>
            <td colspan="12">blanket impl using Polygon</td>
        </tr>
        <tr><td>Rect (Rc)</td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td><td></td></tr>
        <tr>
            <td>MPl</td>
            <td colspan="12">blanket impl using Polygon</td>
        </tr>
        <tr>
            <td>Geometry (Gm)</td>
            <td colspan="12">TODO</td>
        </tr>
        <tr>
            <td>GeometryCollection (GmC)</td>
            <td colspan="12">TODO</td>
        </tr>
    </tbody>
</table>

### Legend

- **Yes** Implemented, and verified to be correct to the best of our knowledge.
- **Bug** Implemented, but known to have some bugs.
- **Sym** Implemented via symmetry
- **TODO** Well, todo.
