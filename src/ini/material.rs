use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

use crate::ini::common::{IdStringParam,
                         ParseSlice,
                         ParseError,
                         ParseResult,
                         chop_param,
                         parse_tokens_with,
                         parse_tokens_strict_with,
                         };

pub type Color = (f32, f32, f32, f32);

pub enum Token<'a> {
    Submaterial(IdStringParam<'a>),
    Texture((u8, IdStringParam<'a>)),
    TextureNoMip((u8, IdStringParam<'a>)),
    TextureMtl((u8, IdStringParam<'a>)),
    TextureNoMipMtl((u8, IdStringParam<'a>)),
    DiffuseColor(Color),
    SpecularColor(Color),
    AmbientColor(Color),
    SpecularPower(f32),
    End,
}


impl<'a> Token<'a> {
    const SUBMATERIAL:       &'static str = "$SUBMATERIAL";
    const TEXTURE:           &'static str = "$TEXTURE";
    const TEXTURE_NOMIP:     &'static str = "$TEXTURE_NOMIP";
    const TEXTURE_MTL:       &'static str = "$TEXTURE_MTL";
    const TEXTURE_NOMIP_MTL: &'static str = "$TEXTURE_NOMIP_MTL";
    const DIFFUSE_COLOR:     &'static str = "$DIFFUSECOLOR";
    const SPECULAR_COLOR:    &'static str = "$SPECULARCOLOR";
    const AMBIENT_COLOR:     &'static str = "$AMBIENTCOLOR";
    const SPECULAR_POWER:    &'static str = "$SPECULARPOWER";
    const END:               &'static str = "$END";

    fn parse(src: &'a str) -> ParseResult<Self> {
        lazy_static! {
            static ref RX_TYPE: Regex = Regex::new(r"^(\$?[0-9A-Z_]+)(\s+(.+))?$").unwrap();
        }

        let (t_type, rest) = chop_param(Some(src), &RX_TYPE).map_err(|e| format!("Cannot parse token type: {}", e))?;
        macro_rules! parse {
            ($id:ident, $t:ty) => {
                <$t>::parse(rest).map(|(p, rest)| (Self::$id(p), rest))
            };
            ($id:ident) => {
                Ok((Self::$id, rest))
            };
        }

        match t_type {
            Self::SUBMATERIAL       => parse!(Submaterial,     IdStringParam),
            Self::TEXTURE           => parse!(Texture,         (u8, IdStringParam)),
            Self::TEXTURE_NOMIP     => parse!(TextureNoMip,    (u8, IdStringParam)),
            Self::TEXTURE_MTL       => parse!(TextureMtl,      (u8, IdStringParam)),
            Self::TEXTURE_NOMIP_MTL => parse!(TextureNoMipMtl, (u8, IdStringParam)),
            Self::DIFFUSE_COLOR     => parse!(DiffuseColor,    Color),
            Self::SPECULAR_COLOR    => parse!(SpecularColor,   Color),
            Self::AMBIENT_COLOR     => parse!(AmbientColor,    Color),
            Self::SPECULAR_POWER    => parse!(SpecularPower,   f32),
            Self::END               => parse!(End),
            _ => Err(format!("Unknown token type: \"{}\"", t_type))
        }
    }
}


impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Submaterial(p)          => write!(f, "{} {}",     Self::SUBMATERIAL,       p),
            Self::Texture((i, p))         => write!(f, "{} {} {}",  Self::TEXTURE,           i, p),
            Self::TextureNoMip((i, p))    => write!(f, "{} {} {}",  Self::TEXTURE_NOMIP,     i, p),
            Self::TextureMtl((i, p))      => write!(f, "{} {} {}",  Self::TEXTURE_MTL,       i, p),
            Self::TextureNoMipMtl((i, p)) => write!(f, "{} {} {}",  Self::TEXTURE_NOMIP_MTL, i, p),
            Self::DiffuseColor(c)         => write!(f, "{} {:?}",   Self::DIFFUSE_COLOR,     c),
            Self::SpecularColor(c)        => write!(f, "{} {:?}",   Self::SPECULAR_COLOR,    c),
            Self::AmbientColor(c)         => write!(f, "{} {:?}",   Self::AMBIENT_COLOR,     c),
            Self::SpecularPower(x)        => write!(f, "{} {}",     Self::SPECULAR_POWER,    x),
            Self::End                     => write!(f, "{}",        Self::END),
        }
    }
}


impl super::IniToken for Token<'_> {
    fn serialize<W: std::io::Write>(&self, mut wr: W) -> Result<(), std::io::Error>{
        match self {
            Self::DiffuseColor((r, g, b, a))  => write!(wr, "{} {} {} {} {}", Self::DIFFUSE_COLOR,  r, g, b, a),
            Self::SpecularColor((r, g, b, a)) => write!(wr, "{} {} {} {} {}", Self::SPECULAR_COLOR, r, g, b, a),
            Self::AmbientColor((r, g, b, a))  => write!(wr, "{} {} {} {} {}", Self::AMBIENT_COLOR,  r, g, b, a),
            s => write!(wr, "{}", s)
        }
    }
}


lazy_static! {
    static ref RX_SPLIT: Regex = Regex::new(r"(?s)(\s*\n)+\s*").unwrap();
}


#[inline]
pub fn parse_tokens<'a>(src: &'a str) -> Vec<(&'a str, ParseResult<'a, Token<'a>>)> {
    parse_tokens_with(src, &RX_SPLIT, Token::parse)
}


#[inline]
pub fn parse_tokens_strict<'a>(src: &'a str) -> Result<Vec<(&'a str, Token<'a>)>, Vec<(&'a str, ParseError)>> {
    parse_tokens_strict_with(src, &RX_SPLIT, Token::parse)
}
