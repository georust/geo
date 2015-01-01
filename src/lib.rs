use std::ascii::AsciiExt;
use tokenizer::{Token, Tokenizer};

mod tokenizer;


pub struct Point {
    x: f64,
    y: f64,
    z: Option<f64>,
    m: Option<f64>,
}

impl Point {
    fn from_tokens(tokens: &mut Tokenizer) ->  Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => return Err("FIXME"),
        };
        let x = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("FIXME"),
        };
        let y = match tokens.next() {
            Some(Token::Number(n)) => n,
            _ => return Err("FIXME"),
        };
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("FIXME"),
        };
        Ok(Point {x: x, y: y, z: None, m: None})
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
            // None
            _ => Err("Invalid WKT format"),
        }
    }
}


#[test]
fn basic_point() {
    let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
    assert_eq!(1, wkt.items.len());
    let point = wkt.items.pop().unwrap();
    assert_eq!(10.0, point.x);
    assert_eq!(-20.0, point.y);
    assert_eq!(None, point.z);
    assert_eq!(None, point.m);
}
