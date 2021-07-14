use std::fmt;
use std::str;


#[derive(Debug)]
pub enum NmfError {
    UnknownHeader,
    SizeMismatch,
    EOF,
    ObjectZeroHeadMissing,
}

pub type ParseResult<'a, T> = Result<(T, &'a [u8]), NmfError>;


#[derive(Debug)]
pub struct Nmf<'a> {
    pub header: NmfHeader,
    pub submaterials: Vec<SubMaterial<'a>>,
    pub objects: Vec<Object<'a>>,

}

#[derive(Debug, Clone, Copy)]
pub enum NmfHeader {
    FromObj,
    B3dmh10
}

#[derive(Debug)]
pub struct SubMaterial<'a> {
    pub slice: &'a [u8],
    pub name: CStrName<'a>
}

#[derive(Debug)]
pub enum CStrName<'a> {
    Valid(&'a str, usize),
    InvalidNotTerminated,
    InvalidEmpty,
    InvalidUtf8(str::Utf8Error)
}

#[derive(Debug)]
pub struct Object<'a> {
    pub slice: &'a [u8],
    pub name: CStrName<'a>,
    pub submaterial_idx: Option<u32>
}


//-------------------------------------------------------------------

impl fmt::Display for Nmf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Header: {}", self.header)?;

        writeln!(f, "Submaterials: {}", self.submaterials.len())?;
        for (i, sm) in self.submaterials.iter().enumerate() {
            writeln!(f, "  {:2}: {:2}", i, sm)?;
        }

        writeln!(f, "Objects: {}", self.objects.len())?;
        for (i, o) in self.objects.iter().enumerate() {
            writeln!(f, "  {:2}: {:2}", i, o)?;
        }

        Ok(())
    }
}

impl fmt::Display for NmfHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NmfHeader::FromObj => write!(f, "fromObj"),
            NmfHeader::B3dmh10 => write!(f, "B3DMH 10")
        }
    }
}

impl fmt::Display for CStrName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            //CStrName::Valid(s, n) => write!(f, "\"{}\" ({} bytes)", s, n),
            CStrName::Valid(s, _) => write!(f, "\"{}\"", s),
            CStrName::InvalidNotTerminated => write!(f, "Invalid (not terminated)"),
            CStrName::InvalidEmpty => write!(f, "Invalid (empty)"),
            CStrName::InvalidUtf8(e) => write!(f, "Invalid (utf-8 error: {})", e)
        }
    }
}


impl fmt::Display for SubMaterial<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Name: {}", self.name)
    }
}


impl fmt::Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(idx) = self.submaterial_idx {
            write!(f, "[{}] ", idx)?;
        }
        
        write!(f, "Name: {}, real size: {} bytes", self.name, self.slice.len())
    }
}





//----------------------------------------------------------------


#[inline]
fn chop_subslice<'a>(slice: &'a [u8], len: usize) -> ParseResult<&'a [u8]> {
    if len > slice.len() {
        Err(NmfError::EOF)
    } else {
        Ok(slice.split_at(len))
    }
}

#[inline]
fn parse_u32<'a>(slice: &'a [u8]) -> ParseResult<u32> {
    let (s, rest) = chop_subslice(slice, 4)?;
    // INVARIANT: s.len() >= 4
    let u: u32 = unsafe { std::mem::transmute_copy(&s[0]) };
    Ok((u, rest))
}

trait FromBytes<'a> {
    fn parse_bytes(b: &'a [u8], nmf_type: NmfHeader) -> ParseResult<'a, Self> where Self: Sized;
}

fn parse_vec<'a, T: FromBytes<'a>>(mut slice: &'a [u8], count: usize, nmf_type: NmfHeader) -> ParseResult<Vec<T>> {
    let mut result = Vec::<T>::with_capacity(count);

    for _ in 0 .. count {
        let (sm, r) = T::parse_bytes(slice, nmf_type)?;
        result.push(sm);
        slice = r;
    }

    Ok((result, slice))
}



//----------------------------------------------------------------

impl<'a> Nmf<'a> {
    pub fn parse_bytes(slice: &'a [u8]) -> ParseResult<Nmf<'a>> {
        
        let (header, rest) = NmfHeader::parse_bytes(slice)?;

        let (submat_count, rest) = parse_u32(rest)?;
        let (obj_count, rest)    = parse_u32(rest)?;
        let (nmf_length, rest)   = parse_u32(rest)?;

        if (nmf_length as usize) != slice.len() {
            return Err(NmfError::SizeMismatch);
        }

        let (submaterials, rest) = parse_vec::<SubMaterial>(rest, submat_count as usize, header)?;
        let (objects, rest) = parse_vec::<Object>(rest, obj_count as usize, header)?;



        Ok((Nmf{ header, submaterials, objects }, rest))
    }
}


impl NmfHeader {
    fn parse_bytes<'a>(b: &'a [u8]) -> ParseResult<'a, NmfHeader> {
        match chop_subslice(b, 8)? {
            (b"fromObj\0", rest) => Ok((NmfHeader::FromObj, rest)),
            (b"B3DMH\010", rest) => Ok((NmfHeader::B3dmh10, rest)),
            _ => Err(NmfError::UnknownHeader)
        }
    }
}


impl<'a> CStrName<'a> {
    fn from_slice(slice: &'a [u8]) -> CStrName<'a> {
        let len = slice.iter().position(|&x| x == 0);
        match len {
            Some(0) => CStrName::InvalidEmpty,
            Some(n) => {
                // INVARIANT: 0 < n < slice length
                let parsed = str::from_utf8(unsafe { slice.get_unchecked(0 .. n) });
                match parsed {
                    Ok(s) => CStrName::Valid(s, n + 1),
                    Err(e) => CStrName::InvalidUtf8(e)
                }
            },
            None => CStrName::InvalidNotTerminated,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            CStrName::Valid(s, _) => Some(s),
            _ => None
        }
    }

}

impl<'a> FromBytes<'a> for SubMaterial<'a> {
    fn parse_bytes(slice: &'a [u8], _: NmfHeader) -> ParseResult<'a, SubMaterial> {
        let (slice, rest) = chop_subslice(slice, 64)?;
        let name = CStrName::from_slice(slice);
        Ok((SubMaterial { slice, name }, rest))
    }
}

impl<'a> FromBytes<'a> for Object<'a> {
    fn parse_bytes(slice: &'a [u8], nmf_type: NmfHeader) -> ParseResult<Object<'a>> {

        // object starts with 4 zero-bytes (0x00_00_00_00) ...
        let (z, rest) = parse_u32(slice)?;
        if z != 0 {
            return Err(NmfError::ObjectZeroHeadMissing)
        }

        let (obj_size, rest) = {
            let (x, rest) = parse_u32(rest)?;
            let x = match nmf_type {
                NmfHeader::FromObj => x,
                // these have weird size error:
                NmfHeader::B3dmh10 => x - 4
            };
            (x as usize, rest)
        };

        let (obj_remainder, rest_of_nmf) = chop_subslice(rest, obj_size - 8)?;

        let submaterial_idx = match nmf_type {
            NmfHeader::FromObj => {
                let y = obj_remainder.len();
                let sm_bytes = obj_remainder.get(y - 4 .. y).unwrap();
                let (i, _) = parse_u32(sm_bytes)?;
                Some(i)
            },
            NmfHeader::B3dmh10 => None
        };

        // TODO: get other object data from the slice
        let name = CStrName::from_slice(obj_remainder);

        Ok((Object { slice: slice.get(0 .. obj_size).unwrap(), name, submaterial_idx }, rest_of_nmf))
    }
}

/*
fn main() {
    

    let filepath = PathBuf::from(r"z:\nmf\0cfddb3cf6d5e2e619f114d288eed911.nmf");
    assert!(filepath.exists());

    let data = fs::read(filepath).unwrap();

    let (nmf, rest) = Nmf::parse_bytes(&data).unwrap();

    println!("{}", nmf);

    assert_eq!(rest.len(), 0);
}*/
