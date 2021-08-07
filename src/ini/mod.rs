use std::io::Write;
use std::fmt;

pub mod building;
use building::Token as BldToken;


pub type ParseError = String;

type ParseResult<'a, T> = Result<(T, Option<&'a str>), ParseError>;


pub trait IniToken<'a>: Sized {
    fn parse_tokens(src: &'a str) -> Vec<(&'a str, ParseResult<'a, Self>)>;
    fn parse_strict(src: &'a str) -> Result<Vec<(&'a str, Self)>, Vec<(&'a str, ParseError)>>;
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

pub struct IniFile<'a, T: IniToken<'a>> {
    ini_slice: &'a str,
    tokens: Vec<(&'a str, IniTokenState<T>)>
}



impl<'a, T> IniFile<'a, T> where T: IniToken<'a> {
    pub fn from_slice(ini_slice: &'a str) -> Result<Self, Vec<(&'a str, ParseError)>> {
        T::parse_strict(ini_slice).map(|tokens| 
            IniFile { 
                ini_slice, 
                tokens: tokens.into_iter()
                              .map(|(chunk, t_val)| (chunk, IniTokenState::Original(t_val)))
                              .collect()
            })
    }

    pub fn write_to<W: Write>(&self, mut wr: W) -> std::io::Result<()> where T: std::fmt::Display {
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


pub type BuildingIni<'a> = IniFile<'a, BldToken<'a>>;

impl BuildingIni<'_> {
    pub fn scale(&mut self, factor: f64) {
        //for (_, t_state) in self.tokens.iter() {
        //    println!("{}", t_state);
        //}

        for (_, t_state) in self.tokens.iter_mut() {
            t_state.modify(|t| t.maybe_scale(factor));
        }

        //println!("======================================");
        //for (_, t_state) in self.tokens.iter() {
        //    println!("{}", t_state);
        //}
    }
}
