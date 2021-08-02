use std::mem::size_of;
use std::alloc;
use std::io::{Read, Seek};
use std::convert::TryInto;
use core::ops::Range;


use super::{ObjectError, ObjectReader, NameBuf};



pub struct ObjectFull {
    head_buf: [u8; 260],
    range_name: Option<Range<usize>>,

    buf_ptr: *mut u8,
    buf_layout: alloc::Layout,

    vertices_count: usize,
    indices_count:  usize,
    faces_count:    usize,
    submat_count:   usize,

    vertices_start:  usize,
    normals_start:   usize,
    uv_map_start:    usize,
    face_ext_start:  usize,
    face_bbox_start: usize,
    submat_start:    usize,
}


#[repr(C)]
pub struct RawFace {
    pub v1: u16,
    pub v2: u16,
    pub v3: u16
}


#[derive(Debug)]
#[repr(C)]
pub struct RawVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32
}


#[derive(Debug)]
#[repr(C)]
pub struct RawPoint {
    pub x: f32,
    pub y: f32,
}


fn read_u32(bytes: &[u8]) -> Result<u32, ObjectError> {
    let ar: [u8; 4] = bytes[0..4].try_into().map_err(|_| ObjectError::SliceReadU32)?;
    Ok(u32::from_le_bytes(ar))
}


fn read_u32size(bytes: &[u8]) -> Result<usize, ObjectError> {
    Ok(read_u32(bytes)? as usize)
}


impl<R: Read + Seek> ObjectReader<R> for ObjectFull {
    fn from_reader(rdr: &mut R) -> Result<ObjectFull, ObjectError> {

        let mut head_buf = [0u8; 260];
        rdr.read_exact(&mut head_buf[..]).map_err(ObjectError::FileIO)?;

        let range_name = {
            let name_start = 8;
            let len = NameBuf::get_len(&head_buf[name_start .. name_start + NameBuf::BUF_LENGTH]);
            if len > 0 {
                Some(name_start .. name_start + len)
            } else {
                None
            }
        };

        
        let vertices_count = read_u32size(&head_buf[236..])?;
        let indices_count  = read_u32size(&head_buf[240..])?;
        let submat_count   = read_u32size(&head_buf[244..])?;
        let faces_count    = get_faces_count(indices_count)?;

        let indices_bytes = indices_count * size_of::<u16>();

        let vertices_start  = indices_bytes   + indices_bytes % size_of::<u32>();
        let normals_start   = vertices_start  + vertices_count * 12;
        let uv_map_start    = normals_start   + vertices_count * 36;
        let face_ext_start  = uv_map_start    + vertices_count * 8;
        let face_bbox_start = face_ext_start  + faces_count * 16;
        let submat_start    = face_bbox_start + faces_count * 24;
        let obj_end         = submat_start    + submat_count * 12;

        unsafe {
            let buf_layout = Self::get_buf_layout(obj_end)?;
            // TODO: without zeroed (MaybeUninit etc)
            let buf_ptr = alloc::alloc_zeroed(buf_layout);
            if buf_ptr.is_null() {
                return Err(ObjectError::Allocation(String::from("Allocated zero pointer")));
            }

            let s1 = std::slice::from_raw_parts_mut(buf_ptr, obj_end);
            rdr.read_exact(&mut s1[0 .. indices_bytes]).map_err(ObjectError::FileIO)
               .and_then(|_| rdr.read_exact(&mut s1[vertices_start .. obj_end]).map_err(ObjectError::FileIO))
               .map_err(|e| {
                    alloc::dealloc(buf_ptr, buf_layout);
                    e
               })?;

            Ok(ObjectFull { head_buf,
                            range_name,

                            buf_ptr,
                            buf_layout,

                            vertices_count,
                            indices_count,
                            faces_count,
                            submat_count,

                            vertices_start,
                            normals_start,
                            uv_map_start,
                            face_ext_start,
                            face_bbox_start,
                            submat_start,
            })

        }

    }
}


impl Drop for ObjectFull {
    fn drop(&mut self) {
        unsafe { 
            alloc::dealloc(self.buf_ptr, self.buf_layout);
        }
    }
}


impl ObjectFull {

    fn get_buf_layout(size: usize) -> Result<alloc::Layout, ObjectError> {
        alloc::Layout::from_size_align(size, 4_usize).map_err(|e| ObjectError::Allocation(format!("{:?}", e)))
    }

    pub fn name(&self) -> &str {
        match &self.range_name {
            Some(rng) => unsafe { std::str::from_utf8_unchecked(self.head_buf.get_unchecked(rng.clone())) },
            None => &"<not displayable>"
        }
    }

    fn get_slice<'a, T>(&'a self, offset: usize, count: usize) -> &'a [T] {
        unsafe {
            let ptr = (self.buf_ptr as *const u8).add(offset).cast::<T>();
            std::slice::from_raw_parts(ptr, count)
        }
    }

    fn get_slice_mut<'a, T>(&'a mut self, offset: usize, count: usize) -> &'a mut [T] {
        unsafe {
            let ptr = self.buf_ptr.add(offset).cast::<T>();
            std::slice::from_raw_parts_mut(ptr, count)
        }
    }

    pub fn vertices<'a>(&'a self) -> &'a [RawVertex] {
        self.get_slice::<RawVertex>(self.vertices_start, self.vertices_count)
    }

    pub fn vertices_mut<'a>(&'a mut self) -> &'a mut [RawVertex] {
        self.get_slice_mut::<RawVertex>(self.vertices_start, self.vertices_count)
    }

    pub fn normals_1<'a>(&'a self) -> &'a [RawVertex] {
        self.get_slice::<RawVertex>(self.normals_start, self.vertices_count)
    }

    pub fn uv_map<'a>(&'a self) -> &'a [RawPoint] {
        self.get_slice::<RawPoint>(self.uv_map_start, self.vertices_count)
    }

    pub fn faces<'a>(&'a self) -> &'a [RawFace] {
        self.get_slice::<RawFace>(0, self.faces_count)
    }
}


#[inline]
fn get_faces_count(indices: usize) -> Result<usize, ObjectError> {
    let (c, rm) = num::integer::div_rem(indices, 3);
    if rm != 0 {
        // TODO: no cast
        return Err(ObjectError::WrongIndicesCount(indices as u32));
    }

    Ok(c)
}
//-----------------------------------------------------------------------------


//-----------------------------------------------------------------------------

/*

#[derive(Debug)]
pub enum ObjectError {
    EOF(usize, ChopEOF, &'static str),
    ZeroHeadMissing,
    Name(ChopEOF),
    WrongIndicesCount(usize),
}
*/

/*
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
*/

/*
//#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RawVertex {
    x: f32,
    y: f32,
    z: f32
}


//#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RawPoint {
    x: f32,
    y: f32,
}


//#[derive(Debug, Clone, Copy)]

//#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RawFaceExtra {
    auto_normal: RawVertex,
    factor: f32
}


//#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct BBox {
    v_min:   RawVertex,
    v_max:   RawVertex,
}


//#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct SubmaterialUsage {
    index_1:  u32,
    index_2:  u32,
    sm_index: u32
}
*/

//----------------------------------------------------------------


/*
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
*/


//----------------------------------------------------------------
/*
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
*/
