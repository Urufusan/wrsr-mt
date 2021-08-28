use std::str::FromStr;
use std::fmt::{Formatter, Error, Display};

use lazy_static::lazy_static;
use regex::Regex;
use const_format::concatcp;


pub type ParseError = String;

pub type ParseResult<'a, T> = Result<(T, Option<&'a str>), ParseError>;

#[derive(Clone)]
pub struct Point3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub struct Rect {
    pub x1: f32,
    pub z1: f32,
    pub x2: f32,
    pub z2: f32
}

pub enum StrValue<'a> {
    Borrowed(&'a str),
    Owned(String),
}

pub struct QuotedStringParam<'a>(pub StrValue<'a>);

pub struct IdStringParam<'a>(pub StrValue<'a>);

impl<'a> IdStringParam<'a> {
    pub fn as_str(&'a self) -> &'a str {
        match &self.0 {
            StrValue::Borrowed(x) => x,
            StrValue::Owned(x) => x.as_str(),
        }
    }

    pub fn new_borrowed(s: &'a str) -> Self {
        IdStringParam(StrValue::Borrowed(s))
    }

    pub fn new_cloned(s: &'_ str) -> Self {
        IdStringParam(StrValue::Owned(s.to_string()))
    }

    pub fn new_owned(s: String) -> Self {
        IdStringParam(StrValue::Owned(s))
    }
}

impl AsRef<str> for IdStringParam<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

pub struct CostKeywordParam<'a>(pub IdStringParam<'a>);
impl<'a> CostKeywordParam<'a> {
    pub fn as_str(&'a self) -> &'a str { self.0.as_str() }
}


//------------------------------------------------------


pub trait ParseSlice<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> where Self: Sized;
}

impl<'a, T1, T2> ParseSlice<'a> for (T1, T2)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        Ok(((t1, t2), src))
    }
}


impl<'a, T1, T2, T3> ParseSlice<'a> for (T1, T2, T3)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>,
      T3: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        let (t3, src) = T3::parse(src)?;
        Ok(((t1, t2, t3), src))
    }
}


impl<'a, T1, T2, T3, T4> ParseSlice<'a> for (T1, T2, T3, T4)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>,
      T3: ParseSlice<'a>,
      T4: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        let (t3, src) = T3::parse(src)?;
        let (t4, src) = T4::parse(src)?;
        Ok(((t1, t2, t3, t4), src))
    }
}


impl ParseSlice<'_> for Point3f {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        let((x, y, z), src) = <(f32, f32, f32) as ParseSlice>::parse(src)?;
        Ok((Point3f { x, y, z }, src))
    }
}


impl ParseSlice<'_> for Rect {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        let((x1, z1, x2, z2), src) = <(f32, f32, f32, f32) as ParseSlice>::parse(src)?;
        Ok((Rect { x1, z1, x2, z2 }, src))
    }
}



impl ParseSlice<'_> for f32 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^(-?[0-9]*\.?[0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| f32::from_str(s).map_err(|e| format!("f32 parse failed: {}", e)))
    }
}


impl ParseSlice<'_> for u8 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| u8::from_str(s).map_err(|e| format!("u8 parse failed: {}", e)))
    }
}


impl ParseSlice<'_> for u32 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| u32::from_str(s).map_err(|e| format!("u32 parse failed: {}", e)))
    }
}


impl<'a> ParseSlice<'a> for QuotedStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!("(?s)^\"([^\"\\n]+)\"", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}


impl<'a> ParseSlice<'a> for IdStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([^[:space:]]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}


impl<'a> ParseSlice<'a> for CostKeywordParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(r"^\$(.+)").unwrap();
        }

        let src = src.ok_or(String::from("Cost keyword parse failed: no data"))?;
        match RX.captures(src) {
            Some(caps) => {
                let rest = caps.get(1).map(|x| x.as_str());
                let (inner, rest) = IdStringParam::parse(rest)?;
                Ok((CostKeywordParam(inner), rest))
            },
            None => Err(format!("Cost keyword must start with '$'. Chunk: [{}]", src))
        }
    }
}

//-----------------------------------------------------------------


impl Display for Point3f {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {}, {})", self.x1, self.z1, self.x2, self.z2)
    }
}

impl Display for QuotedStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "\"{}\"", s)
    }
}

impl Display for IdStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}

impl Display for StrValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s: &str = match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str()
        };

        write!(f, "{}", s)
    }
}

impl Display for CostKeywordParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "${}", s)
    }
}

//--------------------------------------------------------

pub const RX_REMAINDER: &str = r"($|\s*(.*))";


pub fn chop_param<'a, 'b>(src: Option<&'a str>, rx: &'b Regex) -> ParseResult<'a, &'a str> {
    let src = src.ok_or(String::from("Chop param failed: not enough data"))?;

    match rx.captures(src) {
        Some(caps) => {
            // allow panic here: this should not happen with valid regex:
            let t = caps.get(1).expect("Regex is broken").as_str();
            let rest = caps.get(3).map(|x| x.as_str());
            Ok((t, rest))
        },
        None => Err(format!("No match in this chunk: [{}]", src))
    }
}

pub fn parse_param<'a, T, F: Fn(&'a str) -> Result<T, ParseError>>(src: Option<&'a str>, rx: &Regex, f: F) -> ParseResult<'a, T> {
    let (src, rest) = chop_param(src, rx)?;
    let v = f(src).map_err(|e| format!("parse_param failed: {}", e))?;
    Ok((v, rest))
}


//---------------------------------------------------------

impl Point3f {
    pub fn scaled(&self, factor: f64) -> Point3f {
        Point3f {
            x: ((self.x as f64) * factor) as f32,
            y: ((self.y as f64) * factor) as f32,
            z: ((self.z as f64) * factor) as f32,
        }
    }
}

//---------------------------------------------------------

pub fn parse_tokens_with<'a, T, F>(src: &'a str, rx: &Regex, f: F) -> Vec<(&'a str, ParseResult<'a, T>)> 
where F: Fn(&'a str) -> ParseResult<T>
{
    rx.split(src)
        .filter(|x| !x.is_empty())
        .map(|t_str| (t_str, f(t_str)))
        .collect()
}


pub fn parse_tokens_strict_with<'a, T, F>(src: &'a str, rx: &Regex, f: F) -> Result<Vec<(&'a str, T)>, Vec<(&'a str, ParseError)>>
where F: Fn(&'a str) -> ParseResult<T>
{
    let mut res = Vec::with_capacity(100);
    let mut errors = Vec::with_capacity(0);

    for t_str in rx.split(src).filter(|x| !x.is_empty()) {
        match f(t_str) {
            Ok((t_val, rest)) => {
                match rest {
                    Some(r) if !r.is_empty() => {
                        errors.push((t_str, format!("Token parsed incomplete. Remaining: {}", r)));
                    },
                    _ => res.push((t_str, t_val))
                }
            },
            Err(e) => {
                errors.push((t_str, e));
            }
        }
    }

    if errors.is_empty() {
        Ok(res)
    } else {
        Err(errors)
    }
}
