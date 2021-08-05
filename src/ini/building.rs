use std::fs;
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::str::FromStr;

use lazy_static::lazy_static;
use const_format::concatcp;
use regex::Regex;


type ParseError = String;

type ParseResult<'a, T> = Result<(T, Option<&'a str>), ParseError>;


trait ParamParser<'a> {
    type Output;

    fn parse(src: Option<&'a str>) -> ParseResult<Self::Output>;
}

#[derive(Debug)]
struct TokenParams6<'a, P1, P2, P3, P4, P5, P6>
where P1: ParamParser<'a>,
      P2: ParamParser<'a>,
      P3: ParamParser<'a>,
      P4: ParamParser<'a>,
      P5: ParamParser<'a>,
      P6: ParamParser<'a>,
{
    p1: P1::Output,
    p2: P2::Output,
    p3: P3::Output,
    p4: P4::Output,
    p5: P5::Output,
    p6: P6::Output,
}

impl<'a, P1, P2, P3, P4, P5, P6> TokenParams6<'a, P1, P2, P3, P4, P5, P6> 
where P1: ParamParser<'a>, 
      P2: ParamParser<'a>,
      P3: ParamParser<'a>,
      P4: ParamParser<'a>,
      P5: ParamParser<'a>,
      P6: ParamParser<'a>,
{
    fn parse(src: Option<&'a str>) -> ParseResult<TokenParams6<'a, P1, P2, P3, P4, P5, P6>> {
        let (p1, src) = P1::parse(src)?;
        let (p2, src) = P2::parse(src)?;
        let (p3, src) = P3::parse(src)?;
        let (p4, src) = P4::parse(src)?;
        let (p5, src) = P5::parse(src)?;
        let (p6, src) = P6::parse(src)?;

        Ok((TokenParams6 { p1, p2, p3, p4, p5, p6 }, src))
    }
}

impl<'a, P1, P2, P3, P4, P5, P6> Display for TokenParams6<'a, P1, P2, P3, P4, P5, P6>
where P1: ParamParser<'a>, P1::Output: Display,
      P2: ParamParser<'a>, P2::Output: Display,
      P3: ParamParser<'a>, P3::Output: Display,
      P4: ParamParser<'a>, P4::Output: Display,
      P5: ParamParser<'a>, P5::Output: Display,
      P6: ParamParser<'a>, P6::Output: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{{ {} {} {} {} {} {} }}", self.p1, self.p2, self.p3, self.p4, self.p5, self.p6)
    }
    
}


struct ParamNone { }

impl Display for ParamNone {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}


struct NoParamParser { }

impl ParamParser<'_> for NoParamParser {
    type Output = ParamNone;

    fn parse(src: Option<&str>) -> ParseResult<Self::Output> {
        Ok((ParamNone {}, src))
    }
}

struct FloatParamParser { }

impl ParamParser<'_> for FloatParamParser {
    type Output = f32;

    fn parse(src: Option<&str>) -> ParseResult<Self::Output> {
        lazy_static! {
            static ref RX: Regex = Regex::new(r"(?s)^(-?[0-9]*\.?[0-9]+)($|\s*(.*))").unwrap();
        }

        let src = src.ok_or(String::from("Param parse failed: no data"))?;

        match RX.captures(src) {
            Some(caps) => {
                // allow panic here: this should not happen with valid regex:
                let t = caps.get(1).expect("Regex is broken").as_str();
                let rest = caps.get(3).map(|x| x.as_str());

                let f = f32::from_str(t).map_err(|e| format!("Float parse failed: {}", e))?;
                Ok((f, rest))
            },
            None => Err(String::from("Float parse failed"))
        }
    }
}

struct IdentifierParamParser { }

impl<'a> ParamParser<'a> for IdentifierParamParser {
    type Output = &'a str;

    fn parse(src: Option<&'a str>) -> ParseResult<Self::Output> {
        lazy_static! {
            static ref RX: Regex = Regex::new(r"(?s)^([^[:space:]]+)($|\s*(.*))").unwrap();
        }

        let src = src.ok_or(String::from("Param parse failed: no data"))?;

        match RX.captures(src) {
            Some(caps) => {
                // allow panic here: this should not happen with valid regex:
                let t = caps.get(1).expect("Regex is broken").as_str();
                let rest = caps.get(3).map(|x| x.as_str());

                Ok((t, rest))
            },
            None => Err(String::from("Identifier parse failed"))
        }
    }
}



type TokenParams3<'a, P1, P2, P3> = TokenParams6<'a, P1, P2, P3, NoParamParser, NoParamParser, NoParamParser>;
type TokenParams2<'a, P1, P2>     = TokenParams3<'a, P1, P2, NoParamParser>;
type TokenParams1<'a, P1>         = TokenParams2<'a, P1, NoParamParser>;
type TokenParams0<'a>             = TokenParams1<'a, NoParamParser>;


type TokenParams6Floats<'a> = TokenParams6<'a, FloatParamParser, FloatParamParser, FloatParamParser, FloatParamParser, FloatParamParser, FloatParamParser>;

type TokenParamsParticle<'a> = TokenParams6<'a, IdentifierParamParser, FloatParamParser, FloatParamParser, FloatParamParser, FloatParamParser, FloatParamParser>;

enum Token<'a> {

    NameStr(TokenParams0<'a>),
    Name(TokenParams0<'a>),

    TypeLiving(TokenParams0<'a>),

    Storage(TokenParams0<'a>),

    ConnectionPedestrian(TokenParams6Floats<'a>),

    Particle(TokenParamsParticle<'a>),

    CostWork(TokenParams0<'a>),
    CostWorkBuildingNode(TokenParams1<'a, IdentifierParamParser>),
    CostResource(TokenParams0<'a>),
    CostResourceAuto(TokenParams0<'a>),
    CostWorkVehicleStationAccordingNode(TokenParams0<'a>),
}


impl<'a> Token<'a> {
    const NAME_STR: &'static str = "NAME_STR";
    const NAME: &'static str = "NAME";

    const TYPE_LIVING: &'static str = "TYPE_LIVING";

    const STORAGE: &'static str = "STORAGE";

    const CONNECTION_PEDESTRIAN: &'static str = "CONNECTION_PEDESTRIAN";

    const PARTICLE: &'static str = "PARTICLE";

    const COST_WORK: &'static str = "COST_WORK";
    const COST_WORK_BUILDING_NODE: &'static str = "COST_WORK_BUILDING_NODE";
    const COST_RESOURCE: &'static str = "COST_RESOURCE";
    const COST_RESOURCE_AUTO: &'static str = "COST_RESOURCE_AUTO";
    const COST_WORK_VEHICLE_STATION_ACCORDING_NODE: &'static str = "COST_WORK_VEHICLE_STATION_ACCORDING_NODE";

    fn parse(src: &'a str) -> ParseResult<Token<'a>> {
        let (t_type, rest) = chop_token_type(src)?;
        match t_type {
            Self::NAME_STR => 
                TokenParams0::parse(rest).map(|(p, rest)| (Self::NameStr(p), rest)),

            Self::NAME => 
                TokenParams0::parse(rest).map(|(p, rest)| (Self::Name(p), rest)),

            Self::TYPE_LIVING =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::TypeLiving(p), rest)),

            Self::STORAGE =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::Storage(p), rest)),

            Self::CONNECTION_PEDESTRIAN =>
                TokenParams6Floats::parse(rest).map(|(p, rest)| (Self::ConnectionPedestrian(p), rest)),

            Self::PARTICLE =>
                TokenParamsParticle::parse(rest).map(|(p, rest)| (Self::Particle(p), rest)),

            Self::COST_WORK =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::CostWork(p), rest)),

            Self::COST_WORK_BUILDING_NODE =>
                TokenParams1::<IdentifierParamParser>::parse(rest).map(|(p, rest)| (Self::CostWorkBuildingNode(p), rest)),

            Self::COST_RESOURCE =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::CostResource(p), rest)),

            Self::COST_RESOURCE_AUTO =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::CostResourceAuto(p), rest)),

            Self::COST_WORK_VEHICLE_STATION_ACCORDING_NODE =>
                TokenParams0::parse(rest).map(|(p, rest)| (Self::CostWorkVehicleStationAccordingNode(p), rest)),

            _ => Err(format!("Unknown token type: [{}]", t_type))
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Name(p) => write!(f, "{}: {}", Self::NAME, p),
            Self::NameStr(p) => write!(f, "{}: {}", Self::NAME_STR, p),
            Self::TypeLiving(_) => write!(f, "{}", Self::TYPE_LIVING),
            Self::Storage(p) => write!(f, "{}: {}", Self::STORAGE, p),
            Self::ConnectionPedestrian(p) => write!(f, "{}: {}", Self::CONNECTION_PEDESTRIAN, p),
            Self::Particle(p) => write!(f, "{}: {}", Self::PARTICLE, p),
            Self::CostWork(p) => write!(f, "{}: {}", Self::COST_WORK, p),
            Self::CostWorkBuildingNode(p) => write!(f, "{}: {}", Self::COST_WORK_BUILDING_NODE, p),
            Self::CostResource(p) => write!(f, "{}: {}", Self::COST_RESOURCE, p),
            Self::CostResourceAuto(p) => write!(f, "{}: {}", Self::COST_RESOURCE_AUTO, p),
            Self::CostWorkVehicleStationAccordingNode(p) => write!(f, "{}: {}", Self::COST_WORK_VEHICLE_STATION_ACCORDING_NODE, p)
        }
    }
}


fn chop_token_type<'a>(src: &'a str) -> ParseResult<&'a str> {
    lazy_static! {
        static ref RX_TYPE: Regex = Regex::new(r"(?s)^([A-Z_]+)($|\s*(.*))").unwrap();
    }

    match RX_TYPE.captures(src) {
        Some(caps) => {
            // allow panic here: this should not happen with valid regex:
            let t = caps.get(1).expect("Regex is broken").as_str();
            let rest = caps.get(3).map(|x| x.as_str());
            Ok((t, rest))
        },
        None => Err(format!("Cannot parse token type from this: [{}]", src))
    }
}



pub fn do_stuff() {
    let file = fs::read_to_string(r"z:\building.ini").unwrap();

    let tokens = get_tokens(&file);

    for t in tokens {
        //println!("token: [{}]", t);
        match Token::parse(t) {
            Ok((t, rest)) => {
                print!("{}", t);
                if let Some(rest) = rest {
                    print!(" [remainder: {:?}]", rest);
                }
                println!();
            },
            Err(e) => println!("Error: {} [{}]", e, t),
        }
    }

    //println!("OK");
}


fn get_tokens<'a>(src: &'a str) -> Vec<&'a str> {
    const RX_SKIP_LINE: &str = r"((--[^\r\n]*)?\s*\r?\n)+";

    lazy_static! {
        static ref RX_SPLIT: Regex = Regex::new(concatcp!("(", RX_SKIP_LINE, r"|^", RX_SKIP_LINE, r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
    }

    RX_SPLIT.split(src).filter(|x| !x.is_empty()).collect()
}
