// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use tokenizer::PeekableTokens;
use types::FromTokens;
use types::coord::Coord;
use WktItem;


#[derive(Default)]
pub struct LineString {
    pub coords: Vec<Coord>
}

impl LineString {
    pub fn as_item(self) -> WktItem {
        WktItem::LineString(self)
    }
}

impl FromTokens for LineString {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(<Coord as FromTokens>::from_tokens, tokens);
        result.map(|vec| LineString {coords: vec})
    }
}


#[cfg(test)]
mod tests {
    use {Wkt, WktItem};

    #[test]
    fn basic_linestring() {
        let mut wkt = Wkt::from_str("LINESTRING (10 -20, -0 -0.5)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let linestring = match wkt.items.pop().unwrap() {
            WktItem::LineString(linestring) => linestring,
            _ => unreachable!(),
        };
        assert_eq!(2, linestring.coords.len());

        assert_eq!(10.0, linestring.coords[0].x);
        assert_eq!(-20.0, linestring.coords[0].y);
        assert_eq!(None, linestring.coords[0].z);
        assert_eq!(None, linestring.coords[0].m);

        assert_eq!(0.0, linestring.coords[1].x);
        assert_eq!(-0.5, linestring.coords[1].y);
        assert_eq!(None, linestring.coords[1].z);
        assert_eq!(None, linestring.coords[1].m);
    }
}
