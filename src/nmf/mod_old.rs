use std::str;

mod display;
pub mod slices;


#[derive(Debug)]
pub struct Nmf<'a> {
    slice: &'a [u8],
    header: Header,
    submaterials: Vec<Submaterial<'a>>,
    objects: Vec<Object<'a>>,
}


#[derive(Debug, Clone, Copy)]
enum Header {
    FromObj,
    B3dmh10
}


#[derive(Debug)]
pub enum Name<'a> {
    Valid(&'a str, usize),
    Empty,
    InvalidUtf8(&'a [u8], str::Utf8Error)
}


#[derive(Debug)]
pub struct Submaterial<'a> {
    slice: &'a [u8],
    pub name: Name<'a>
}


#[derive(Debug)]
struct Object<'a> {
    slice: &'a [u8],
    name: Name<'a>,
    submaterials: Vec<usize>
}


/*
#[derive(Debug)] 
pub enum NmfModifier {
    RemoveObject(String),
    PruneSubmaterials,
    Transform(GeometryModifier)
}


#[derive(Debug)]
pub enum GeometryModifier {
    Scale(f64),
    MirrorX,
    MirrorZ,
    Offset(f64, f64),
    Rotate(f64),
}
*/

#[derive(Debug)]
pub enum NmfError {
    UnknownHeader,
    NoSubmaterials,
    NoObjects,
    SizeMismatch,
    EOF,
    ObjectZeroHeadMissing,
}


pub type ParseResult<'a, T> = Result<(T, &'a [u8]), NmfError>;


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
fn chop_u32(slice: &[u8]) -> ParseResult<u32> {
    let (s, rest) = chop_subslice(slice, 4)?;
    // INVARIANT: s.len() >= 4
    let u: u32 = unsafe { std::mem::transmute_copy(&s[0]) };
    Ok((u, rest))
}


fn parse_vec<'a, T, F>(mut slice: &'a [u8], count: usize, parser: F) -> ParseResult<Vec<T>>
where F: Fn(&'a [u8]) -> ParseResult<T>,
      T: 'a
{
    let mut result = Vec::<T>::with_capacity(count);

    for _i in 0 .. count {
        // NOTE: debug
        // println!("parse vec {}", _i);
        let (sm, r) = parser(slice)?;
        result.push(sm);
        slice = r;
    }

    Ok((result, slice))
}


//----------------------------------------------------------------


#[inline]
fn write_usize32<T>(i: usize, wr: &mut T)
where T: std::io::Write
{
    assert!(i <= u32::MAX as usize, "Too big usize to be written as u32");

    wr.write_all(
        unsafe {
            std::mem::transmute::<&usize, &[u8; 4]>(&i)
        }
    ).unwrap();
}


//----------------------------------------------------------------

impl<'a> Nmf<'a> {
    pub fn parse_bytes(slice: &'a [u8]) -> ParseResult<Nmf<'a>> {
        
        let (header, rest) = Header::parse_bytes(slice)?;

        let (submat_count, rest) = chop_u32(rest)?;
        if submat_count == 0 {
            return Err(NmfError::NoSubmaterials);
        }

        let (obj_count, rest) = chop_u32(rest)?;
        if obj_count == 0 {
            return Err(NmfError::NoObjects);
        }

        let (nmf_length, rest) = chop_u32(rest)?;
        let nmf_length = nmf_length as usize;

        if nmf_length != slice.len() {
            return Err(NmfError::SizeMismatch);
        }

        let (submaterials, rest) = parse_vec(rest, submat_count as usize, Submaterial::parse_bytes)?;
        let (objects, rest) = parse_vec(rest, obj_count as usize, Object::parse_bytes)?;

        Ok((Nmf{ slice: &slice[0 .. nmf_length], header, submaterials, objects }, rest))
    }


    pub fn len(&self) -> usize {
        self.slice.len()
    }

/*
    fn modified_len(&self) -> usize {
        match self.modifiers
        8 // header
        + 4 // submaterial count
        + 4 // objects count
        + 4 // nmf length
        + 64 * self.submaterials.len()
        + self.objects.iter().fold(0_usize, |sz, obj| sz + obj.slice.len())
    }*/


    pub fn write_bytes<T>(&self, wr: &mut T) 
    where T: std::io::Write {
        self.header.write_bytes(wr);
        write_usize32(self.submaterials.len(), wr);
        write_usize32(self.objects.len(), wr);
        write_usize32(self.len(), wr);

        for sm in self.submaterials.iter() {
            wr.write_all(sm.slice).unwrap();
            sm.slice.len();
        }

        for obj in self.objects.iter() {
            obj.write_bytes(wr);
            obj.slice.len();
        }
    }


    pub fn get_unused_submaterials(&'a self) -> impl Iterator<Item = &'a Submaterial> + 'a {
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


impl Header {
    const FROM_OBJ: &'static [u8] = b"fromObj\0";
    const B3DMH_10: &'static [u8] = b"B3DMH\010";

    fn parse_bytes<'a>(b: &'a [u8]) -> ParseResult<'a, Header> {
        match chop_subslice(b, 8)? {
            (Self::FROM_OBJ, rest) => Ok((Header::FromObj, rest)),
            (Self::B3DMH_10, rest) => Ok((Header::B3dmh10, rest)),
            _ => Err(NmfError::UnknownHeader)
        }
    }

    fn write_bytes<T>(&self, wr: &mut T)
    where T: std::io::Write {
        let bytes = match self {
            Header::FromObj => Self::FROM_OBJ,
            Header::B3dmh10 => Self::B3DMH_10
        };

        wr.write_all(bytes).unwrap();
    }
}


impl<'a> Name<'a> {
    fn from_slice(slice: &'a [u8]) -> Name<'a> {
        let len = slice.iter().position(|&x| x == 0).unwrap_or(slice.len());

        if len == 0 {
            Name::Empty
        } else {
            // INVARIANT: 0 < len <= slice length
            let slice = unsafe { slice.get_unchecked(0 .. len) };
            let parsed = str::from_utf8(slice);
            match parsed {
                Ok(s) => Name::Valid(s, len),
                Err(e) => Name::InvalidUtf8(slice, e)
            }
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Name::Valid(s, _) => Some(s),
            _ => None
        }
    }

}

impl<'a> Submaterial<'a> {
    fn parse_bytes(slice: &'a [u8]) -> ParseResult<'a, Submaterial> {
        let (slice, rest) = chop_subslice(slice, 64)?;
        let name = Name::from_slice(slice);
        Ok((Submaterial { slice, name }, rest))
    }
}

impl<'a> Object<'a> {
    fn parse_bytes(slice: &'a [u8]) -> ParseResult<Object<'a>> {

        // object starts with 4 zero-bytes (0x00_00_00_00) ...
        let (z, obj_rest) = chop_u32(slice)?;
        if z != 0 {
            return Err(NmfError::ObjectZeroHeadMissing)
        }

        let (obj_size1, obj_rest) = chop_u32(obj_rest)?;

        let (name_slice, obj_rest) = chop_subslice(obj_rest, 64)?;
        let name = Name::from_slice(name_slice);

        // Next bytes after the object name:
        //
        // (4) magic u32: for B3DMH_10 = 0xFFFF_0000
        //                    fromObj  = 0x0000_0000
        //
        // (128) matrix-like bytes (f32)
        // (24) min-max coords (2 x 3 x f32)
        // (4) magic 1_u32
        // (4) additional size (= main size - 228)
        
        let (_, obj_rest) = chop_subslice(obj_rest, 160)?;
        let (obj_size2, _) = chop_u32(obj_rest)?;

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
                let (sm_idx, _) = chop_u32(&sm_tail[idx_offset .. ])?;
                idx_offset += 12;
                submaterials.push(sm_idx as usize);
            }

            submaterials
        };

        Ok((Object { slice: obj_slice, name, submaterials }, nmf_rest))
    }

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
        let (c, _rest) = chop_u32(&slice[244..])?;
        Ok(c as usize)
    }

/*
    pub fn count_submaterials(&self) -> Result<usize, NmfError> {
        Self::get_submaterials(&self.slice[244..])
    }
*/    
}
