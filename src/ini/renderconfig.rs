use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

use crate::ini::common::{Point3f, 
                         IdStringParam,
                         ParseSlice,
                         ParseError,
                         ParseResult,
                         chop_param,
                         parse_tokens_with,
                         parse_tokens_strict_with,
                         };

pub type LightColor = (f32, f32, f32);

pub enum Token<'a> {
    End,
    ObjectTypeStock(IdStringParam<'a>),
    ObjectTypeWorkshop,
    Model(IdStringParam<'a>),
    ModelLod((IdStringParam<'a>, f32)),
    ModelLod2((IdStringParam<'a>, f32)),
    ModelEmissive(IdStringParam<'a>),
    Material(IdStringParam<'a>),
    MaterialEmissive(IdStringParam<'a>),
    PlaneShadow,
    Reflection,
    ExactSpecular,
    FieldCollision,
    VariableMaterialParams,
    SmokePointChance(f32),
    Life(f32),
    ExplosionGroup(u32),
    DerbisFallingFx((IdStringParam<'a>, f32)),
    DerbisFalledFx((IdStringParam<'a>, f32)),
    DerbisFalledSfx(IdStringParam<'a>),
    DerbisNum(u32),
    DerbisFallingFxMaxTime(f32),
    DerbisScale(f32),
    DerbisMesh((IdStringParam<'a>, IdStringParam<'a>)),
    Light((Point3f, f32)),
    LightRgb((Point3f, f32, LightColor)),
    LightRgbBlink((Point3f, f32, LightColor)),
}

impl<'a> Token<'a> {
    const END:                       &'static str = "END";
    const TYPE_STOCK:                &'static str = "$TYPE";
    const TYPE_WORKSHOP:             &'static str = "$TYPE_WORKSHOP";
    const MODEL:                     &'static str = "MODEL";
    const MODEL_LOD:                 &'static str = "MODEL_LOD";
    const MODEL_LOD2:                &'static str = "MODEL_LOD2";
    const MODEL_EMISSIVE:            &'static str = "MODELEMISSIVE";
    const MATERIAL:                  &'static str = "MATERIAL";
    const MATERIAL_EMISSIVE:         &'static str = "MATERIALEMISSIVE";
    const PLANE_SHADOW:              &'static str = "PLANESHADOW";
    const REFLECTION:                &'static str = "REFLECTION";
    const EXACT_SPECULAR:            &'static str = "EXACTSPECULAR";
    const FIELD_COLLISION:           &'static str = "FIELDCOLLISION";
    const VARIABLE_MATERIAL_PARAMS:  &'static str = "VARIABLEMATERIALPARAMS";
    const SMOKEPOINT_CHANCE:         &'static str = "SMOKEPOINTCHANCE";
    const LIFE:                      &'static str = "LIFE";
    const EXPLOSION_GROUP:           &'static str = "EXPLOSION_GROUP";
    const DERBIS_FALLING_FX:         &'static str = "DERBIS_FALLING_FX";
    const DERBIS_FALLED_FX:          &'static str = "DERBIS_FALLED_FX";
    const DERBIS_FALLED_SFX:         &'static str = "DERBIS_FALLED_SFX";
    const DERBIS_NUM:                &'static str = "DERBIS_NUM";
    const DERBIS_FALLING_FX_MAXTIME: &'static str = "DERBIS_FALLING_FX_MAXTIME";
    const DERBIS_SCALE:              &'static str = "DERBIS_SCALE";
    const DERBIS_MESH:               &'static str = "DERBIS_MESH";
    const LIGHT:                     &'static str = "LIGHT";
    const LIGHT_RGB:                 &'static str = "LIGHT_RGB";
    const LIGHT_RGB_BLINK:           &'static str = "LIGHT_RGB_BLICK";

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
            Self::END                       => parse!(End),
            Self::TYPE_STOCK                => parse!(ObjectTypeStock,    IdStringParam),
            Self::TYPE_WORKSHOP             => parse!(ObjectTypeWorkshop),
            Self::MODEL                     => parse!(Model,              IdStringParam),
            Self::MODEL_LOD                 => parse!(ModelLod,           (IdStringParam, f32)),
            Self::MODEL_LOD2                => parse!(ModelLod2,          (IdStringParam, f32)),
            Self::MODEL_EMISSIVE            => parse!(ModelEmissive,      IdStringParam),
            Self::MATERIAL                  => parse!(Material,           IdStringParam),
            Self::MATERIAL_EMISSIVE         => parse!(MaterialEmissive,   IdStringParam),
            Self::PLANE_SHADOW              => parse!(PlaneShadow),
            Self::REFLECTION                => parse!(Reflection),
            Self::EXACT_SPECULAR            => parse!(ExactSpecular),
            Self::FIELD_COLLISION           => parse!(FieldCollision),
            Self::VARIABLE_MATERIAL_PARAMS  => parse!(VariableMaterialParams),
            Self::SMOKEPOINT_CHANCE         => parse!(SmokePointChance,   f32),
            Self::LIFE                      => parse!(Life,               f32),
            Self::EXPLOSION_GROUP           => parse!(ExplosionGroup,     u32),
            Self::LIGHT                     => parse!(Light,              (Point3f, f32)),
            Self::LIGHT_RGB                 => parse!(LightRgb,           (Point3f, f32, LightColor)),
            Self::LIGHT_RGB_BLINK           => parse!(LightRgbBlink,      (Point3f, f32, LightColor)),
            Self::DERBIS_FALLING_FX         => parse!(DerbisFallingFx,    (IdStringParam, f32)),
            Self::DERBIS_FALLED_FX          => parse!(DerbisFalledFx,     (IdStringParam, f32)),
            Self::DERBIS_FALLED_SFX         => parse!(DerbisFalledSfx,    IdStringParam),
            Self::DERBIS_NUM                => parse!(DerbisNum,          u32),
            Self::DERBIS_SCALE              => parse!(DerbisScale,        f32),
            Self::DERBIS_MESH               => parse!(DerbisMesh,         (IdStringParam, IdStringParam)),
            Self::DERBIS_FALLING_FX_MAXTIME => parse!(DerbisFallingFxMaxTime, f32),
            _ => Err(format!("Unknown token type: \"{}\"", t_type))
        }
    }
}


impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::End                         => write!(f, "{}",       Self::END),
            Self::ObjectTypeStock(p)          => write!(f, "{} {}",    Self::TYPE_STOCK, p),
            Self::ObjectTypeWorkshop          => write!(f, "{}",       Self::TYPE_WORKSHOP),
            Self::Model(p)                    => write!(f, "{} {}",    Self::MODEL, p),
            Self::ModelLod((p, x))            => write!(f, "{} {} {}", Self::MODEL_LOD, p, x),
            Self::ModelLod2((p, x))           => write!(f, "{} {} {}", Self::MODEL_LOD2, p, x),
            Self::ModelEmissive(p)            => write!(f, "{} {}",    Self::MODEL_EMISSIVE, p),
            Self::Material(p)                 => write!(f, "{} {}",    Self::MATERIAL, p),
            Self::MaterialEmissive(p)         => write!(f, "{} {}",    Self::MATERIAL_EMISSIVE, p),
            Self::PlaneShadow                 => write!(f, "{}",       Self::PLANE_SHADOW),
            Self::Reflection                  => write!(f, "{}",       Self::REFLECTION),
            Self::ExactSpecular               => write!(f, "{}",       Self::EXACT_SPECULAR),
            Self::FieldCollision              => write!(f, "{}",       Self::FIELD_COLLISION),
            Self::VariableMaterialParams      => write!(f, "{}",       Self::VARIABLE_MATERIAL_PARAMS),
            Self::SmokePointChance(x)         => write!(f, "{} {}",    Self::SMOKEPOINT_CHANCE, x),
            Self::Life(x)                     => write!(f, "{} {}",    Self::LIFE, x),
            Self::ExplosionGroup(n)           => write!(f, "{} {}",    Self::EXPLOSION_GROUP, n),
            Self::DerbisFallingFx((p, x))     => write!(f, "{} {} {}", Self::DERBIS_FALLING_FX, p, x),
            Self::DerbisFalledFx((p, x))      => write!(f, "{} {} {}", Self::DERBIS_FALLED_FX, p, x),
            Self::DerbisFalledSfx(p)          => write!(f, "{} {}",    Self::DERBIS_FALLED_SFX, p),
            Self::DerbisNum(n)                => write!(f, "{} {}",    Self::DERBIS_NUM, n),
            Self::DerbisFallingFxMaxTime(x)   => write!(f, "{} {}",    Self::DERBIS_FALLING_FX_MAXTIME, x),
            Self::DerbisScale(x)              => write!(f, "{} {}",    Self::DERBIS_SCALE, x),
            Self::DerbisMesh((p1, p2))        => write!(f, "{} {} {}", Self::DERBIS_MESH, p1, p2),
            Self::Light((pt, x))              => write!(f, "{} {} {}", Self::LIGHT, pt, x),
            Self::LightRgb((pt, x, col))      => write!(f, "{} {} {} {:?}", Self::LIGHT_RGB, pt, x, col),
            Self::LightRgbBlink((pt, x, col)) => write!(f, "{} {} {} {:?}", Self::LIGHT_RGB_BLINK, pt, x, col),
        }
    }
}


impl super::IniToken for Token<'_> {
    fn serialize<W: std::io::Write>(&self, mut wr: W) -> Result<(), std::io::Error>{
        match self {
            Self::Light((pt, x))                    => write!(wr, "{} {} {} {} {}", Self::LIGHT, pt.x, pt.y, pt.z, x),
            Self::LightRgb((pt, x, (r, g, b)))      => write!(wr, "{} {} {} {} {} {} {} {}", Self::LIGHT_RGB, pt.x, pt.y, pt.z, x, r, g, b),
            Self::LightRgbBlink((pt, x, (r, g, b))) => write!(wr, "{} {} {} {} {} {} {} {}", Self::LIGHT_RGB_BLINK, pt.x, pt.y, pt.z, x, r, g, b),
            s => write!(wr, "{}", s)
        }
    }
}


lazy_static! {
    static ref RX_SPLIT: Regex = Regex::new(r"(?s)(^\s|(\s*\n)+)\s*").unwrap();
}


#[inline]
pub fn parse_tokens<'a>(src: &'a str) -> Vec<(&'a str, ParseResult<'a, Token<'a>>)> {
    parse_tokens_with(src, &RX_SPLIT, Token::parse)
}


#[inline]
pub fn parse_tokens_strict<'a>(src: &'a str) -> Result<Vec<(&'a str, Token<'a>)>, Vec<(&'a str, ParseError)>> {
    parse_tokens_strict_with(src, &RX_SPLIT, Token::parse)
}
