use std::fmt;

pub mod modify;


pub struct NmfSlice<'a> {
    slice: &'a [u8],

    header_type: HeaderType,
    submaterials: Vec<Name<'a>>,
    objects: Vec<ObjectSlice<'a>>
}


enum HeaderType {
    FromObj,
    B3dmh10
}


struct Name<'a> {
    bytes: &'a [u8],
    displayed: Option<&'a str>
}


pub type NmfResult<'a, T> = Result<(T, &'a [u8]), NmfError<'a>>;


#[derive(Debug)]
pub enum NmfError<'a> {
    HeaderEOF(usize, ChopEOF),
    HeaderType(&'a [u8]),
    NoSubmaterials,
    NoObjects,
    SliceTooShort(usize, usize),

    Submaterial(usize, ChopEOF),
    Object(usize, ObjectError)
}


#[derive(Debug)]
pub enum ObjectError {
    EOF(usize, ChopEOF, &'static str),
    ZeroHeadMissing,
    Name(ChopEOF),
    WrongIndicesCount(usize),
}



struct ObjectSlice<'a> {
    slice:   &'a [u8],
    
    size_1:  usize,
    name:    Name<'a>,
    magic_1: &'a [u8],
    bbox:    BBox,
    magic_2: u32,
    size_2:  usize,
    magic_3: &'a [u8],

    indices:     &'a [FaceIndices],
    vertices:    &'a [Vertex3f],

    normals:     &'a [Vertex3f],
    tangents_1:  &'a [Vertex3f],
    tangents_2:  &'a [Vertex3f],

    uv_map:      &'a [Point2f],
    face_extra:  &'a [FaceData],
    face_bboxes: &'a [BBox],

    submaterials: &'a [SubmaterialUsage]
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct Vertex3f {
    x: f32,
    y: f32,
    z: f32
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct Point2f {
    x: f32,
    y: f32,
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct FaceIndices {
    v1: u16,
    v2: u16,
    v3: u16
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct FaceData {
    auto_normal: Vertex3f,
    factor: f32
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct BBox {
    v_min:   Vertex3f,
    v_max:   Vertex3f,
}


//#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
struct SubmaterialUsage {
    index_1:  u32,
    index_2:  u32,
    sm_index: u32
}


//----------------------------------------------------------------


impl<'a> NmfSlice<'a> {

    pub fn parse_slice(slice: &'a [u8]) -> NmfResult<NmfSlice<'a>> {

        let slice_len = slice.len();

        let (header_type, rest) = HeaderType::parse_slice(slice)?;

        let (submat_count, rest) = chop_u32_usize(rest).map_err(|e| NmfError::HeaderEOF(slice_len - rest.len(), e))?;
        if submat_count == 0 {
            return Err(NmfError::NoSubmaterials);
        }

        let (obj_count, rest) = chop_u32_usize(rest).map_err(|e| NmfError::HeaderEOF(slice_len - rest.len(), e))?;
        if obj_count == 0 {
            return Err(NmfError::NoObjects);
        }

        let (nmf_len, rest) = chop_u32_usize(rest).map_err(|e| NmfError::HeaderEOF(slice_len - rest.len(), e))?;
        if slice_len < nmf_len {
            return Err(NmfError::SliceTooShort(slice_len, nmf_len));
        }

        let (submaterials, rest) = chop_vec(rest, submat_count, Name::parse_slice).map_err(|(i, e)| NmfError::Submaterial(i, e))?;
        let (objects, rest) = chop_vec(rest, obj_count, ObjectSlice::parse_slice).map_err(|(i, e)| NmfError::Object(i, e))?;

        Ok((NmfSlice{ slice, header_type, submaterials, objects }, rest))
    }
}


impl<'a> Name<'a> {
    pub fn parse_slice(slice: &'a [u8]) -> ChopResult<Name<'a>> {
        let (bytes, rest) = chop_subslice(slice, 64)?;
        let displayed = {
            let len = bytes.iter().position(|&x| x == 0).unwrap_or(bytes.len());

            if len == 0 {
                None
            } else {
                let s = unsafe { bytes.get_unchecked(0 .. len) };
                std::str::from_utf8(s).ok()
            }
        };

        Ok((Name { bytes, displayed }, rest))

    }
}


impl<'a> ObjectSlice<'a> {

    pub fn parse_slice(slice: &'a [u8]) -> Result<(ObjectSlice<'a>, &'a [u8]), ObjectError> {
        let slice_len = slice.len();

        let (zerohead, rest) = chop_u32(slice).map_err(|e| ObjectError::EOF(0, e, "zero header"))?;
        if zerohead != 0 {
            return Err(ObjectError::ZeroHeadMissing);
        }

        let (size_1, rest)  = chop_u32_usize(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "size 1"))?;
        //let (name, rest)    = chop_subslice(rest, 64).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "name"))?;
        let (name, rest)    = Name::parse_slice(rest).map_err(ObjectError::Name)?;
        let (magic_1, rest) = chop_subslice(rest, 132).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "magic 1"))?; // 4 bytes + some matrix-like structure

        let (bbox, rest)    = chop_as::<BBox>(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "bbox"))?;
        let (magic_2, rest) = chop_u32(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "magic 2"))?;

        let (size_2, rest)        = chop_u32_usize(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "size 2"))?;
        let (vert_count, rest)    = chop_u32_usize(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "vertices count"))?;
        let (indices_count, rest) = chop_u32_usize(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "indices count"))?;
        let (submat_count, rest)  = chop_u32_usize(rest).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "submaterials count"))?;

        let faces_count = {
            let (c, rm) = num::integer::div_rem(indices_count, 3_usize);
            if rm != 0 {
                return Err(ObjectError::WrongIndicesCount(indices_count));
            }

            c
        };

        let (magic_3, rest) = chop_subslice(rest, 12).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "magic 3"))?; // always  0x00 00 00 00   3A 01 04 00   00 00 00 00

        let (indices, rest)      = chop_slice_of::<FaceIndices>(rest, faces_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "indices"))?;
        let (vertices, rest)     = chop_slice_of::<Vertex3f>(rest, vert_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "vertices"))?;
        let (normals, rest)      = chop_slice_of::<Vertex3f>(rest, vert_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "normals"))?;
        let (tangents_1, rest)   = chop_slice_of::<Vertex3f>(rest, vert_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "tangents 1"))?;
        let (tangents_2, rest)   = chop_slice_of::<Vertex3f>(rest, vert_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "tangents 2"))?;
        let (uv_map, rest)       = chop_slice_of::<Point2f>(rest, vert_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "uv map"))?;
        let (face_extra, rest)   = chop_slice_of::<FaceData>(rest, faces_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "face extras"))?;
        let (face_bboxes, rest)  = chop_slice_of::<BBox>(rest, faces_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "face bboxes"))?;
        let (submaterials, rest) = chop_slice_of::<SubmaterialUsage>(rest, submat_count).map_err(|e| ObjectError::EOF(slice_len - rest.len(), e, "submaterials"))?;

        // TODO: compare read-length with size1 and size2

        Ok((ObjectSlice {
            slice,
            
            size_1, name, magic_1,
            bbox, magic_2, size_2, magic_3,
            indices, vertices,
            normals, tangents_1, tangents_2,
            uv_map,
            face_extra, face_bboxes,
            submaterials
        }, rest))
    }

}


impl HeaderType {
    const FROM_OBJ: &'static [u8] = b"fromObj\0";
    const B3DMH_10: &'static [u8] = b"B3DMH\010";

    fn parse_slice<'a>(b: &'a [u8]) -> NmfResult<'a, HeaderType> {
        match chop_subslice(b, 8) {
            Ok((Self::FROM_OBJ, rest)) => Ok((HeaderType::FromObj, rest)),
            Ok((Self::B3DMH_10, rest)) => Ok((HeaderType::B3dmh10, rest)),
            Ok((b, _)) => Err(NmfError::HeaderType(b)),
            Err(e) => Err(NmfError::HeaderEOF(0, e))
        }
    }
}


//----------------------------------------------------------------

#[derive(Debug)]
pub struct ChopEOF {
    need: usize,
    have: usize
}


pub type ChopResult<'a, T> = Result<(T, &'a [u8]), ChopEOF>;


#[inline]
fn chop_subslice<'a>(slice: &'a [u8], len: usize) -> ChopResult<&'a [u8]> {
    if len > slice.len() {
        Err(ChopEOF { need: len, have: slice.len() })
    } else {
        Ok(slice.split_at(len))
    }
}


#[inline]
fn chop_as<T>(slice: &[u8]) -> ChopResult<T> {
    let (s, rest) = chop_subslice(slice, std::mem::size_of::<T>())?;
    // INVARIANT: s.len() === size_of::<T>()
    let result: T = unsafe { std::mem::transmute_copy(&s[0]) };
    Ok((result, rest))
}


#[inline]
fn chop_u32(slice: &[u8]) -> ChopResult<u32> {
    chop_as::<u32>(slice)
}


#[inline]
fn chop_u32_usize(slice: &[u8]) -> ChopResult<usize> {
   chop_u32(slice).map(|(x, rest)| (x as usize, rest)) 
}


#[inline]
fn chop_slice_of<'a, T>(slice: &'a [u8], len: usize) -> ChopResult<&'a [T]> {
    let (s, rest) = chop_subslice(slice, len * std::mem::size_of::<T>())?;
    // INVARIANT: s.len() >= len
    let ptr: *const u8 = &s[0];
    let result = unsafe { std::slice::from_raw_parts(ptr as *const T, len) };
    Ok((result, rest))
}


#[inline]
fn chop_vec<'a, T, F, E>(mut slice: &'a [u8], count: usize, parser: F) -> Result<(Vec<T>, &'a [u8]), (usize, E)>
where F: Fn(&'a [u8]) -> Result<(T, &'a [u8]), E>,
      T: 'a
{
    let mut result = Vec::<T>::with_capacity(count);

    for i in 0 .. count {
        let (t, rest) = parser(slice).map_err(|e| (i, e))?;
        result.push(t);
        slice = rest;
    }

    Ok((result, slice))
}


//----------------------------------------------------------------

/*
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
*/

//--------------------------------------------------------------------

impl fmt::Display for NmfSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Type: {}", self.header_type)?;

        writeln!(f, "Submaterials: {}", self.submaterials.len())?;
        for (i, sm) in self.submaterials.iter().enumerate() {
            writeln!(f, "{:2}) {}", i, sm)?;
        }

        writeln!(f, "Objects: {}", self.objects.len())?;

        for (i, o) in self.objects.iter().enumerate() {
            write!(f, "{:2}) v: {:5}, f: {:5}, sm: [", i, o.vertices.len(), o.indices.len())?;
            let mut ism = o.submaterials.iter();
            if let Some(sm) = ism.next() {
                write!(f, "{}", {sm.sm_index})?;
            }

            while let Some(sm) = ism.next() {
                write!(f, ";{}", {sm.sm_index})?;
            }

            writeln!(f, "], \"{}\"", &o.name)?;
        }

        Ok(())
    }
}


impl fmt::Display for HeaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            HeaderType::FromObj => write!(f, "fromObj"),
            HeaderType::B3dmh10 => write!(f, "B3DMH 10")
        }
    }
}


impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.displayed {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "<not displayable>")
        }
    }
}        
