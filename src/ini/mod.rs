pub mod building;

pub type ParseError = String;
pub type ParseResult<'a, T> = Result<(T, Option<&'a str>), ParseError>;



pub trait IniToken<'a>: Sized {
    fn parse_tokens(src: &'a str) -> Vec<(&'a str, ParseResult<'a, Self>)>;
    fn parse_strict(src: &'a str) -> Result<Vec<(&'a str, Self)>, Vec<(&'a str, ParseError)>>;
}


pub struct IniFile<'a, T: IniToken<'a>> {
    ini_slice: &'a str,
    tokens: Vec<(&'a str, T)>
}



impl<'a, T> IniFile<'a, T>
where T: IniToken<'a>
{
    pub fn from_slice(ini_slice: &'a str) -> Result<Self, Vec<(&'a str, ParseError)>> {
        T::parse_strict(ini_slice).map(|tokens| IniFile { ini_slice, tokens })
    }
}


//type BuildingIniView<'a> = IniView<'a, building::Token<'a>>;


