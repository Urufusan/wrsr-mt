use std::io::Write;
use std::fmt;

pub mod common;

pub mod building;
pub mod renderconfig;
pub mod material;

pub mod scale;

use common::{ParseError, ParseResult};


//---------------------------------------------


pub trait IniToken: Sized {
    fn serialize<W: Write>(&self, wr: W) -> std::io::Result<()>;
}


pub enum IniTokenState<T> {
    Original(T),
    Modified(T)
}


impl<T> IniTokenState<T> {
    fn token(&self) -> &T {
        match self {
            Self::Original(t) => t,
            Self::Modified(t) => t
        }
    }

    fn modify<F: Fn(&T) -> Option<T>>(&mut self, f: F) {
        match f(self.token()) {
            None => { },
            Some(t) => *self = Self::Modified(t)
        }
    }
}


impl<T: fmt::Display> fmt::Display for IniTokenState<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Original(t) => write!(f, "{}", t),
            Self::Modified(t) => write!(f, "* {}", t),
        }
    }
}


//-------------------------------------------------

pub struct IniFile<'a, T: IniToken> {
    ini_slice: &'a str,
    tokens: Vec<(&'a str, IniTokenState<T>)>
}


impl<'a, T> IniFile<'a, T> where T: IniToken {

    pub fn from_parts(ini_slice: &'a str, tokens: Vec<(&'a str, T)>) -> Self {
        IniFile { 
            ini_slice, 
            tokens: tokens.into_iter()
                          .map(|(chunk, t_val)| (chunk, IniTokenState::Original(t_val)))
                          .collect()
        }
    }

    pub fn tokens(&self) -> impl Iterator<Item = &T> {
        self.tokens.iter().map(|(_, t)| t.token())
    }


    pub fn write_to<W: Write>(&self, mut wr: W) -> std::io::Result<()> {
        unsafe {
            // replace 'modified' tokens, dump other stuff as is
            let mut chunk_start = self.ini_slice.as_ptr();

            for (t_str, t_state) in self.tokens.iter() {
                match t_state {
                    IniTokenState::Original(_) => { },
                    IniTokenState::Modified(t) => {
                        let chunk_end = t_str.as_ptr();
                        let chunk_len = chunk_end.offset_from(chunk_start);
                        if chunk_len > 0 {
                            // can write original chunk
                            let chunk = std::slice::from_raw_parts(chunk_start, chunk_len as usize);
                            wr.write_all(chunk)?;

                            chunk_start = chunk_end.add(t_str.len());
                        }

                        t.serialize(&mut wr)?;
                    }
                }
            }

            let written = chunk_start.offset_from(self.ini_slice.as_ptr());
            assert!(written >= 0);
            let written = written as usize;
            assert!(written <= self.ini_slice.len());
            let remaining = self.ini_slice.len() - written;
            if remaining > 0 {
                let last_chunk = std::slice::from_raw_parts(chunk_start, remaining);
                wr.write_all(last_chunk)?;
            }
        } //unsafe

        Ok(())
    }
}


pub type BuildingToken<'a> = building::Token<'a>;
pub type BuildingIni<'a> = IniFile<'a, BuildingToken<'a>>;
pub use building::parse_tokens as parse_building_tokens;

pub fn parse_building_ini<'a>(src: &'a str) -> Result<BuildingIni<'a>, Vec<(&'a str, ParseError)>> {
    building::parse_tokens_strict(src).map(|tokens| BuildingIni::from_parts(src, tokens))
}


pub type RenderToken<'a> = renderconfig::Token<'a>;
pub type RenderIni<'a> = IniFile<'a, RenderToken<'a>>;
pub use renderconfig::parse_tokens as parse_render_tokens;

pub fn parse_renderconfig_ini<'a>(src: &'a str) -> Result<RenderIni<'a>, Vec<(&'a str, ParseError)>> {
    renderconfig::parse_tokens_strict(src).map(|tokens| RenderIni::from_parts(src, tokens))
}


pub type MaterialToken<'a> = material::Token<'a>;
pub type MaterialMtl<'a> = IniFile<'a, MaterialToken<'a>>;
pub use material::parse_tokens as parse_material_tokens;

pub fn parse_mtl<'a>(src: &'a str) -> Result<MaterialMtl<'a>, Vec<(&'a str, ParseError)>> {
    material::parse_tokens_strict(src).map(|tokens| MaterialMtl::from_parts(src, tokens))
}
