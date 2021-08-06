pub mod building;

pub struct ReplacementToken<T> {
    buf: String,
    token: T,
}

pub enum IniToken<'a, T> {
    Original(&'a str, T),
    Modified(&'a str, ReplacementToken<T>),
}


struct IniView<'a, T> {
    ini_slice: &'a str,
    tokens: Vec<(&'a str, T)>
}

type BuildingIniView<'a> = IniView<'a, building::Token<'a>>;
