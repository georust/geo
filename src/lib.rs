use std::ascii::AsciiExt;
use tokenizer::{Token, Tokenizer};

mod tokenizer;


pub struct Coord {
    x: f64,
    y: f64,
    z: Option<f64>,
    m: Option<f64>,
}

impl Coord {
    fn from_tokens(tokens: &mut Tokenizer) ->  Result<Self, &'static str> {
        let x = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("FIXME"),
        };
        let y = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("FIXME"),
        };
        Ok(Coord {x: x, y: y, z: None, m: None})
    }
}


pub struct Point {
    coord: Coord
}

impl Point {
    fn from_tokens(tokens: &mut Tokenizer) ->  Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => return Err("FIXME"),
        };
        let coord = match Coord::from_tokens(tokens) {
            Ok(c) => c,
            Err(s) => return Err(s),
        };
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("FIXME"),
        };
        Ok(Point {coord: coord})
    }
}


pub struct Wkt {
    items: Vec<Point>
}

impl Wkt {
    fn new() -> Self {
        Wkt {items: vec![]}
    }

    fn add_point(&mut self, point: Point) {
        self.items.push(point);
    }

    fn from_reader(reader: &mut Reader) -> Result<Self, &'static str> {
        match reader.read_to_string() {
            Ok(string) => Wkt::from_str(string.as_slice()),
            Err(err) => Err(err.desc),
        }
    }

    fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = tokenizer::tokenize(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(mut tokens: Tokenizer) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                let uppercased = word.to_ascii_uppercase();
                match uppercased.as_slice() {
                    "POINT" => {
                        match Point::from_tokens(&mut tokens) {
                            Ok(point) => {
                                wkt.add_point(point);
                                Ok(wkt)
                            }
                            Err(s) => Err(s),
                        }
                    },
                    _ => Ok(wkt),
                }
            },
            None => Ok(wkt),
            _ => Err("Invalid WKT format"),
        }
    }
}


#[test]
fn empty_string() {
    let mut wkt = Wkt::from_str("").ok().unwrap();
    assert_eq!(0, wkt.items.len());
}


#[test]
fn basic_point() {
    let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
    assert_eq!(1, wkt.items.len());
    let point = wkt.items.pop().unwrap();
    assert_eq!(10.0, point.coord.x);
    assert_eq!(-20.0, point.coord.y);
    assert_eq!(None, point.coord.z);
    assert_eq!(None, point.coord.m);
}


#[test]
fn invalid_points() {
    Wkt::from_str("POINT ()").err().unwrap();
    Wkt::from_str("POINT (10)").err().unwrap();
    Wkt::from_str("POINT 10").err().unwrap();
    Wkt::from_str("POINT (10 -20 40)").err().unwrap();
}
