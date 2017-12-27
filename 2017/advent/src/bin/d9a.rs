#![feature(slice_patterns)]

#[macro_use]
extern crate nom;

use std::io::{stdin, Read};
use std::string;
use nom::{anychar, IResult};

named!(parse<&[u8], Vec<Object>>,
    ws!(many1!(object)));

named!(object<&[u8], Object>,
    terminated!(
        alt!(group | garbage),
        opt!(tag!(","))));

named_args!(objects<'a>(terminator: &'a str) <Vec<Object>>,
    map!(
        many_till!(
            object,
            peek!(tag!(terminator))),
        |(v, _)| v));

named!(group<&[u8], Object>,
    map!(
        delimited!(
            tag!("{"),
            call!(objects, "}"),
            tag!("}")),
        Object::Group));

named!(garbage<&[u8], Object>,
    map_res!(
        delimited!(
            tag!("<"),
            escaped_transform!(
                take_until_either!("!>"),
                '!',
                map!(take!(1), |_| &[])),
            tag!(">")),
        |s| string(s).map(Object::Garbage)));

fn string(u8s: Vec<u8>) -> Result<String, string::FromUtf8Error> {
    String::from_utf8(u8s)
}

fn main() {
    let mut in_ = String::new();
    stdin().read_to_string(&mut in_).unwrap();
    let in_bytes = in_.as_bytes();

    let out_ = parse(&in_bytes);
    let groups = match out_ {
        Ok((&[], o)) => o,
        _ => return,
    };
    println!("in={:?}\nout={:?}", in_, groups);
    let stream = Stream {
        object: groups[0].clone(),
    };
    println!("{:?}", stream);
    println!("{:?}", stream.score());
    println!("{:?}", stream.length());
}

#[derive(Debug, Clone)]
struct Stream {
    object: Object,
}

impl Stream {
    fn score(&self) -> u64 {
        self.object.score(1)
    }

    fn length(&self) -> u64 {
        self.object.length()
    }
}

#[derive(Debug, Clone)]
enum Object {
    Group(Vec<Object>),
    Garbage(String),
}

impl Object {
    fn score(&self, nesting_level: u64) -> u64 {
        match *self {
            Object::Group(ref objects) => {
                nesting_level
                    + objects
                        .iter()
                        .map(|o| o.score(nesting_level + 1))
                        .sum::<u64>()
            }
            Object::Garbage(_) => 0,
        }
    }

    fn length(&self) -> u64 {
        match *self {
            Object::Group(ref objects) => objects.iter().map(Object::length).sum(),
            Object::Garbage(ref s) => s.len() as u64,
        }
    }
}
