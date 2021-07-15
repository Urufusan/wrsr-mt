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

#[derive(Debug, Clone)]
pub struct SubMaterial<'a> {
    pub slice: &'a [u8],
    pub name: CStrName<'a>
}

#[derive(Debug, Clone)]
pub enum CStrName<'a> {
    Valid(&'a str, usize),
    InvalidNotTerminated,
    InvalidEmpty,
    InvalidUtf8(str::Utf8Error)
}

#[derive(Debug, Clone)]
pub struct Object<'a> {
    slice: &'a [u8],
    pub name: CStrName<'a>,
    pub submaterial_idx: Option<usize>
}


//-------------------------------------------------------------------

impl fmt::Display for Nmf<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Header: {}", self.header)?;

        writeln!(f, "Submaterials: {}", self.submaterials.len())?;
        for (i, sm) in self.submaterials.iter().enumerate() {
            writeln!(f, "{:2}) {:2}", i, sm)?;
        }

        writeln!(f, "Objects: {}", self.objects.len())?;
        for (i, o) in self.objects.iter().enumerate() {
            writeln!(f, "{:2}) {:2}", i, o)?;
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
        write!(f, "{}", self.name)
    }
}


impl fmt::Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(idx) = self.submaterial_idx {
            write!(f, "submtl: {:2},  ", idx)?;
        }
        
        write!(f, "size: {:7},  name: {}", self.slice.len(), self.name)
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

#[inline]
fn write_usize32<T>(i: usize, wr: &mut T)
where T: std::io::Write
{
    assert!(i <= u32::MAX as usize);

    wr.write_all(
        unsafe {
            std::mem::transmute::<&usize, &[u8; 4]>(&i)
        }
    ).unwrap();
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

    pub fn write_bytes<T>(&self, wr: &mut T) 
    where T: std::io::Write {
        let nmf_len = 8 // header
                    + 4 // submaterial count
                    + 4 // objects count
                    + 4 // nmf length
                    + 64 * self.submaterials.len()
                    + self.objects.iter().fold(0_usize, |sz, obj| sz + obj.slice.len());
        
        self.header.write_bytes(wr);
        write_usize32(self.submaterials.len(), wr);
        write_usize32(self.objects.len(), wr);
        write_usize32(nmf_len, wr);

        for sm in self.submaterials.iter() {
            wr.write_all(sm.slice).unwrap();
            sm.slice.len();
        }

        for obj in self.objects.iter() {
            obj.write_bytes(wr, self.header);
            obj.slice.len();
        }
    }

    pub fn get_unused_submaterials(&'a self) -> impl Iterator<Item = &'a SubMaterial> + 'a {
        self.submaterials
            .iter()
            .enumerate()
            .filter_map(move |(i, sm)| 
                if self.objects.iter().all(|o| 
                    o.submaterial_idx.map(|idx| idx != i).unwrap_or(true)) 
                {
                    Some(sm)
                } else {
                    None
                })
    }

    pub fn get_used_submaterials(&'a self) -> impl Iterator<Item = &'a SubMaterial> + 'a  {
        self.submaterials
            .iter()
            .enumerate()
            .filter_map(move |(i, sm)| 
                if self.objects.iter().any(|o| 
                    o.submaterial_idx.map(|idx| idx == i).unwrap_or(false)) 
                {
                    Some(sm)
                } else {
                    None
                })
    }
}


impl NmfHeader {
    const FROM_OBJ: &'static [u8] = b"fromObj\0";
    const B3DMH_10: &'static [u8] = b"B3DMH\010";

    fn parse_bytes<'a>(b: &'a [u8]) -> ParseResult<'a, NmfHeader> {
        match chop_subslice(b, 8)? {
            (Self::FROM_OBJ, rest) => Ok((NmfHeader::FromObj, rest)),
            (Self::B3DMH_10, rest) => Ok((NmfHeader::B3dmh10, rest)),
            _ => Err(NmfError::UnknownHeader)
        }
    }

    fn write_bytes<T>(&self, wr: &mut T)
    where T: std::io::Write {
        let bytes = match self {
            NmfHeader::FromObj => Self::FROM_OBJ,
            NmfHeader::B3dmh10 => Self::B3DMH_10
        };

        wr.write_all(bytes).unwrap();
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
                Some(i as usize)
            },
            NmfHeader::B3dmh10 => None
        };

        // TODO: get other object data from the slice
        let name = CStrName::from_slice(obj_remainder);

        Ok((Object { slice: slice.get(0 .. obj_size).unwrap(), name, submaterial_idx }, rest_of_nmf))
    }
}


impl Object<'_> {
    fn write_bytes<T>(&self, wr: &mut T, nmf_type: NmfHeader)
    where T: std::io::Write
    {

        // object starts with 4 zero-bytes (0x00_00_00_00) ...
        wr.write_all(&[0u8; 4]).unwrap();

        match nmf_type {
            NmfHeader::FromObj => {
                let sz = self.slice.len();
                write_usize32(sz, wr);
                wr.write_all(&self.slice[8 .. sz - 4]).unwrap();
                write_usize32(self.submaterial_idx.unwrap(), wr);
            },
            // these have weird size error:
            NmfHeader::B3dmh10 => {
                write_usize32(self.slice.len() + 4, wr);
                wr.write_all(&self.slice[8 .. ]).unwrap();
            }
        };
    }
}
