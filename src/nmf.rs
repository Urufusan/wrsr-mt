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
//    InvalidNotTerminated,
    InvalidEmpty,
    InvalidUtf8(str::Utf8Error)
}

#[derive(Debug, Clone)]
pub struct Object<'a> {
    slice: &'a [u8],
    pub name: CStrName<'a>,
    pub submaterials: Vec<usize>
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
            CStrName::Valid(s, _) => write!(f, "\"{}\"", s),
            //CStrName::InvalidNotTerminated => write!(f, "Invalid (not terminated)"),
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
        write!(f, "submtl: ")?;

        if self.submaterials.is_empty() {
            write!(f, "<NONE>")?;
        } else {
            write!(f, "[")?;
            let mut i = self.submaterials.iter();
            let fst = i.next().unwrap();
            write!(f, "{}", fst)?;

            while let Some(x) = i.next() {
                write!(f, ", {}", x)?;
            }

            write!(f, "]")?;
        };
        
        write!(f, ", size: {:7},  name: {}", self.slice.len(), self.name)
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
    fn parse_bytes(b: &'a [u8]) -> ParseResult<'a, Self> where Self: Sized;
}

fn parse_vec<'a, T: FromBytes<'a>>(mut slice: &'a [u8], count: usize) -> ParseResult<Vec<T>> {
    let mut result = Vec::<T>::with_capacity(count);

    for _i in 0 .. count {
        // NOTE: debug
        // println!("parse vec {}", _i);
        let (sm, r) = T::parse_bytes(slice)?;
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
        assert!(submat_count > 0);
        let (obj_count, rest)    = parse_u32(rest)?;
        assert!(obj_count > 0);
        let (nmf_length, rest)   = parse_u32(rest)?;

        if (nmf_length as usize) != slice.len() {
            return Err(NmfError::SizeMismatch);
        }

        let (submaterials, rest) = parse_vec::<SubMaterial>(rest, submat_count as usize)?;
        let (objects, rest) = parse_vec::<Object>(rest, obj_count as usize)?;



        Ok((Nmf{ header, submaterials, objects }, rest))
    }

    pub fn calculated_len(&self) -> usize {
        8 // header
        + 4 // submaterial count
        + 4 // objects count
        + 4 // nmf length
        + 64 * self.submaterials.len()
        + self.objects.iter().fold(0_usize, |sz, obj| sz + obj.slice.len())
    }

    pub fn write_bytes<T>(&self, wr: &mut T) 
    where T: std::io::Write {
        self.header.write_bytes(wr);
        write_usize32(self.submaterials.len(), wr);
        write_usize32(self.objects.len(), wr);
        write_usize32(self.calculated_len(), wr);

        for sm in self.submaterials.iter() {
            wr.write_all(sm.slice).unwrap();
            sm.slice.len();
        }

        for obj in self.objects.iter() {
            obj.write_bytes(wr);
            obj.slice.len();
        }
    }

    pub fn get_unused_submaterials(&'a self) -> impl Iterator<Item = &'a SubMaterial> + 'a {
        self.submaterials
            .iter()
            .enumerate()
            .filter_map(move |(i, sm)| 
                if self.objects.iter().all(|o| 
                    o.submaterials.iter().all(|&idx| idx != i)) 
                {
                    Some(sm)
                } else {
                    None
                })
    }
/*
    pub fn get_used_submaterials(&'a self) -> impl Iterator<Item = &'a SubMaterial> + 'a  {
        self.submaterials
            .iter()
            .enumerate()
            .filter_map(move |(i, sm)| 
                if self.objects.iter().any(|o| 
                    o.submaterials.iter().any(|&idx| idx == i)) 
                {
                    Some(sm)
                } else {
                    None
                })
    }
*/    
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
        let len = slice.iter().position(|&x| x == 0).unwrap_or(slice.len());

        if len == 0 {
            CStrName::InvalidEmpty
        } else {
            // INVARIANT: 0 < len <= slice length
            let parsed = str::from_utf8(unsafe { slice.get_unchecked(0 .. len) });
            match parsed {
                Ok(s) => CStrName::Valid(s, len),
                Err(e) => CStrName::InvalidUtf8(e)
            }
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
    fn parse_bytes(slice: &'a [u8]) -> ParseResult<'a, SubMaterial> {
        let (slice, rest) = chop_subslice(slice, 64)?;
        let name = CStrName::from_slice(slice);
        Ok((SubMaterial { slice, name }, rest))
    }
}

impl<'a> FromBytes<'a> for Object<'a> {
    fn parse_bytes(slice: &'a [u8]) -> ParseResult<Object<'a>> {

        // object starts with 4 zero-bytes (0x00_00_00_00) ...
        let (z, obj_rest) = parse_u32(slice)?;
        if z != 0 {
            return Err(NmfError::ObjectZeroHeadMissing)
        }

        let (obj_size1, obj_rest) = parse_u32(obj_rest)?;

        let (name_slice, obj_rest) = chop_subslice(obj_rest, 64)?;
        let name = CStrName::from_slice(name_slice);

        // Next bytes after the object name:
        //
        // (4) magic 0_u32
        // (128) matrix-like bytes (f32)
        // (24) min-max coords (2 x 3 x f32)
        // (4) magic 1_u32
        // (4) additional size (= main size - 228)
        
        let (_, obj_rest) = chop_subslice(obj_rest, 160)?;
        let (obj_size2, _) = parse_u32(obj_rest)?;

        let obj_size = (obj_size1 + obj_size1 - obj_size2 - 232) as usize;
        /*
        // keep this for now
        let obj_size = match nmf_type{
            NmfHeader::FromObj => obj_size1,
            NmfHeader::B3dmh10 => obj_size1 + obj_size1 - obj_size2 - 232
        } as usize;
        */

        let (obj_slice, nmf_rest) = chop_subslice(slice, obj_size)?;

        // (4) verts count
        // (4) indices count
        // (4) submaterials count
        // (12) magic bytes 00 00 00 00 3A 01 04 00 00 00 00 00

        // Then variable-length arrays of data:
        //
        // Indices (2 bytes each - u16)
        // Vertices (12 bytes each - 3xf32)
        // smth normals-related (vertices count * 36)
        // UV map (for each vert 2xf32)
        // some weird geometry

        // Then trailing submaterials-usage bytes:
        // 12 bytes x "submaterials count"
        // 12 bytes are u32(indices start?) u32(indices end) u32(submaterial index)

        let submaterials = {
            let sm_count = Self::get_submaterials(slice)?;
            assert!(sm_count > 0, "Object uses 0 submaterials");
            let mut submaterials = Vec::with_capacity(sm_count);

            let sm_tail = {
                let tail_start = obj_slice.len() - 12 * sm_count;
                &obj_slice[tail_start .. ]
            };

            let mut idx_offset = 8;

            for _ in 0 .. sm_count {
                let (sm_idx, _) = parse_u32(&sm_tail[idx_offset .. ])?;
                idx_offset += 12;
                submaterials.push(sm_idx as usize);
            }

            submaterials
        };

        Ok((Object { slice: obj_slice, name, submaterials }, nmf_rest))
    }
}


impl Object<'_> {
    // NOTE: this only overrides submaterials usage, the rest is dumped as original slice
    fn write_bytes<T>(&self, wr: &mut T)
    where T: std::io::Write
    {
        let sm_count = self.submaterials.len();
        let mut sm_start = self.slice.len() - 12 * sm_count;

        wr.write_all(&self.slice[0 .. sm_start]).unwrap();

        for sm in self.submaterials.iter() {
            // boundaries
            wr.write_all(&self.slice[sm_start .. sm_start + 8]).unwrap();
            // submaterial index
            write_usize32(*sm, wr);
            sm_start += 12;
        }
    }
/*
    pub fn count_vertices(&self) -> Result<usize, NmfError> {
        let (c, _rest) = parse_u32(&self.slice[236..])?;
        Ok(c as usize)
    }

    pub fn count_indices(&self) -> Result<usize, NmfError> {
        let (c, _rest) = parse_u32(&self.slice[240..])?;
        Ok(c as usize)
    }
*/
    fn get_submaterials(slice: &[u8]) -> Result<usize, NmfError> {
        let (c, _rest) = parse_u32(&slice[244..])?;
        Ok(c as usize)
    }
/*
    pub fn count_submaterials(&self) -> Result<usize, NmfError> {
        Self::get_submaterials(&self.slice[244..])
    }
*/    
}
