use std::mem::size_of;
use std::alloc;
use std::io::{Write, Read, Seek};
use std::convert::TryInto;
use core::ops::Range;


use super::{ObjectError, ObjectReader, NameBuf};



#[repr(C)]
pub struct ObjectFull {
    head_buf: [u8; 260],
    range_name: Option<Range<usize>>,

    buf_ptr: *mut u8,
    buf_layout: alloc::Layout,

    vertices_count: usize,
    indices_count:  usize,
    faces_count:    usize,
    submat_count:   usize,

    vertices_start:    usize,
    normals1_start:    usize,
    normals2_start:    usize,
    normals3_start:    usize,
    uv_map_start:      usize,
    face_ext_start:    usize,
    face_bboxes_start: usize,
    submat_start:      usize,
}


#[repr(C)]
pub struct RawFace {
    pub v1: u16,
    pub v2: u16,
    pub v3: u16
}


#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct RawVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32
}


#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct RawPoint {
    pub x: f32,
    pub y: f32,
}


#[repr(C)]
pub struct RawFaceExtra {
    pub auto_normal: RawVertex,
    pub factor: f32
}


#[repr(C)]
pub struct RawBBox {
    pub v_min: RawVertex,
    pub v_max: RawVertex,
}

/*
#[repr(C)]
struct SubmaterialUsage {
    index_1:  u32,
    index_2:  u32,
    sm_index: u32
}
*/




fn read_u32(bytes: &[u8]) -> Result<u32, ObjectError> {
    let ar: [u8; 4] = bytes[0..4].try_into().map_err(|_| ObjectError::SliceReadU32)?;
    Ok(u32::from_le_bytes(ar))
}


fn read_u32size(bytes: &[u8]) -> Result<usize, ObjectError> {
    Ok(read_u32(bytes)? as usize)
}


impl<R: Read + Seek> ObjectReader<R> for ObjectFull {
    fn from_reader(rdr: &mut R, _max_sm_idx: usize) -> Result<ObjectFull, ObjectError> {

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

        // possible round-up to 4 byte alignment for the following elements
        let vertices_start    = indices_bytes     + indices_bytes % size_of::<u32>();
        let normals1_start    = vertices_start    + vertices_count * 12;
        let normals2_start    = normals1_start    + vertices_count * 12;
        let normals3_start    = normals2_start    + vertices_count * 12;
        let uv_map_start      = normals3_start    + vertices_count * 12;
        let face_ext_start    = uv_map_start      + vertices_count * 8;
        let face_bboxes_start = face_ext_start    + faces_count    * 16;
        let submat_start      = face_bboxes_start + faces_count    * 24;
        let obj_end           = submat_start      + submat_count   * 12;

        unsafe {
            let buf_layout = alloc::Layout::from_size_align(obj_end, 4_usize).map_err(|e| ObjectError::Allocation(format!("{:?}", e)))?;
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
                            normals1_start,
                            normals2_start,
                            normals3_start,
                            uv_map_start,
                            face_ext_start,
                            face_bboxes_start,
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

    pub fn write_bytes<W: Write>(&self, mut wr: W) -> Result<(), std::io::Error> {
        wr.write_all(&self.head_buf)?;

        let slice = self.get_slice::<u8>(0, self.indices_count * size_of::<u16>());
        wr.write_all(slice)?;

        let slice = self.get_slice::<u8>(self.vertices_start,  self.vertices_count * size_of::<RawVertex>());
        wr.write_all(slice)?;
        let slice = self.get_slice::<u8>(self.normals1_start,  self.vertices_count * size_of::<RawVertex>());
        wr.write_all(slice)?;
        let slice = self.get_slice::<u8>(self.normals2_start,  self.vertices_count * size_of::<RawVertex>());
        wr.write_all(slice)?;
        let slice = self.get_slice::<u8>(self.normals3_start,  self.vertices_count * size_of::<RawVertex>());
        wr.write_all(slice)?;
        let slice = self.get_slice::<u8>(self.uv_map_start,    self.vertices_count * size_of::<RawPoint>());
        wr.write_all(slice)?;

        let slice = self.get_slice::<u8>(self.face_ext_start, self.buf_layout.size() - self.face_ext_start);
        wr.write_all(slice)
    }

    pub fn name(&self) -> &str {
        match &self.range_name {
            Some(rng) => unsafe { std::str::from_utf8_unchecked(self.head_buf.get_unchecked(rng.clone())) },
            None => &"<not displayable>"
        }
    }

    fn bbox_mut<'a>(&'a mut self) -> &'a mut RawBBox {
        unsafe {
            let ptr = self.head_buf.as_mut_ptr().add(204).cast::<RawBBox>();
            ptr.as_mut().unwrap()
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

    pub fn faces<'a>(&'a self) -> &'a [RawFace] {
        self.get_slice::<RawFace>(0, self.faces_count)
    }

    pub fn faces_mut<'a>(&'a mut self) -> &'a mut [RawFace] {
        self.get_slice_mut::<RawFace>(0, self.faces_count)
    }

    pub fn vertices<'a>(&'a self) -> &'a [RawVertex] {
        self.get_slice::<RawVertex>(self.vertices_start, self.vertices_count)
    }

    pub fn vertices_mut<'a>(&'a mut self) -> &'a mut [RawVertex] {
        self.get_slice_mut::<RawVertex>(self.vertices_start, self.vertices_count)
    }

    pub fn normals_1<'a>(&'a self) -> &'a [RawVertex] {
        self.get_slice::<RawVertex>(self.normals1_start, self.vertices_count)
    }

    pub fn normals_1_mut<'a>(&'a mut self) -> &'a mut [RawVertex] {
        self.get_slice_mut::<RawVertex>(self.normals1_start, self.vertices_count)
    }

    //pub fn normals_2<'a>(&'a self) -> &'a [RawVertex] {
        //self.get_slice::<RawVertex>(self.normals2_start, self.vertices_count)
    //}
  
    pub fn normals_2_mut<'a>(&'a mut self) -> &'a mut [RawVertex] {
        self.get_slice_mut::<RawVertex>(self.normals2_start, self.vertices_count)
    }

    //pub fn normals_3<'a>(&'a self) -> &'a [RawVertex] {
        //self.get_slice::<RawVertex>(self.normals3_start, self.vertices_count)
    //}

    pub fn normals_3_mut<'a>(&'a mut self) -> &'a mut [RawVertex] {
        self.get_slice_mut::<RawVertex>(self.normals3_start, self.vertices_count)
    }

    pub fn uv_map<'a>(&'a self) -> &'a [RawPoint] {
        self.get_slice::<RawPoint>(self.uv_map_start, self.vertices_count)
    }

    pub fn face_extras_mut<'a>(&'a mut self) -> &'a mut [RawFaceExtra] {
        self.get_slice_mut::<RawFaceExtra>(self.face_ext_start, self.faces_count)
    }

    pub fn face_bboxes_mut<'a>(&'a mut self) -> &'a mut [RawBBox] {
        self.get_slice_mut::<RawBBox>(self.face_bboxes_start, self.faces_count)
    }

    pub fn scale(&mut self, scale_factor: f64) {
        self.bbox_mut().scale(scale_factor);

        for v in self.vertices_mut() {
            v.scale(scale_factor);
        }

        for RawFaceExtra { factor, .. } in self.face_extras_mut() {
            *factor = (*factor as f64 * scale_factor) as f32;
        }

        for bbox in self.face_bboxes_mut() {
            bbox.scale(scale_factor); 
        }
    }

    pub fn offset(&mut self, dx: f32, dy: f32, dz: f32) {
        self.bbox_mut().offset(dx, dy, dz);

        for v in self.vertices_mut() {
            v.offset(dx, dy, dz);
        }

        for RawFaceExtra { auto_normal: RawVertex { x, y, z },  factor } in self.face_extras_mut() {
            *factor -= *x * dx + *y * dy + *z * dz;
        }

        for bbox in self.face_bboxes_mut() {
            bbox.offset(dx, dy, dz);
        }
    }

    pub fn mirror_z(&mut self) {
        self.bbox_mut().mirror_z();

        for f in self.faces_mut() {
            f.reverse();
        }

        for v in self.vertices_mut() {
            v.mirror_z();
        }

        for n in self.normals_1_mut() {
            n.mirror_z();
        }

        for n in self.normals_2_mut() {
            n.mirror_z();
        }

        for n in self.normals_3_mut() {
            n.mirror_z();
        }

        for RawFaceExtra { auto_normal, .. } in self.face_extras_mut() {
            auto_normal.mirror_z();
        }

        for bbox in self.face_bboxes_mut() {
            bbox.mirror_z();
        }
    }


    pub fn optimize_indices(&mut self) {

        assert!(self.vertices_count < u16::MAX.into());

        let vx_count = self.vertices_count as u16;

        let mut new_verts = ahash::AHashMap::<(RawVertex, RawVertex, RawVertex, RawVertex, RawPoint), u16>::with_capacity(self.vertices_count);
        let mut remap = Vec::<u16>::with_capacity(self.vertices_count);
        let mut kept = 0u16;

        unsafe {

            macro_rules! advance { ($p:ident) => { $p = $p.add(1)} }

            macro_rules! mk_ptr {
                ($t:ty, $ofs:expr) => {{
                    let walker = (self.buf_ptr as *const u8).add($ofs).cast::<$t>();
                    let last   = self.buf_ptr.add($ofs).cast::<$t>();
                    (walker, last)
                }};
            }

            let (mut vx_walk, mut vx_last) = mk_ptr!(RawVertex, self.vertices_start);
            let (mut n1_walk, mut n1_last) = mk_ptr!(RawVertex, self.normals1_start);
            let (mut n2_walk, mut n2_last) = mk_ptr!(RawVertex, self.normals2_start);
            let (mut n3_walk, mut n3_last) = mk_ptr!(RawVertex, self.normals3_start);
            let (mut uv_walk, mut uv_last) = mk_ptr!(RawPoint,  self.uv_map_start);

            for i in 0 .. vx_count {
                let key = (vx_walk.read(), n1_walk.read(), n2_walk.read(), n3_walk.read(), uv_walk.read());
                advance!(vx_walk);
                advance!(n1_walk);
                advance!(n2_walk);
                advance!(n3_walk);
                advance!(uv_walk);

                if !new_verts.contains_key(&key) {
                    if i != kept {
                        vx_last.write(key.0.clone());
                        n1_last.write(key.1.clone());
                        n2_last.write(key.2.clone());
                        n3_last.write(key.3.clone());
                        uv_last.write(key.4.clone());
                    }

                    new_verts.insert(key, kept);

                    advance!(vx_last);
                    advance!(n1_last);
                    advance!(n2_last);
                    advance!(n3_last);
                    advance!(uv_last);

                    remap.push(kept);
                    kept += 1;
                } else {
                    remap.push(new_verts[&key]);
                }
            }

            let removed_verts = self.vertices_count - kept as usize;
            if removed_verts > 0 {
                self.vertices_count = kept as usize;
                (&mut self.head_buf[236..240]).write_all(&kept.to_le_bytes()[..]).unwrap();

                for idx in self.get_slice_mut::<u16>(0, self.indices_count) {
                    *idx = remap[*idx as usize];
                }

                let removed_bytes = (removed_verts * (4 * size_of::<RawVertex>() + size_of::<RawPoint>())) as u32;

                let mut sz = read_u32(&self.head_buf[4..]).unwrap();
                sz -= removed_bytes;
                (&mut self.head_buf[4..]).write_all(&sz.to_le_bytes()[..]).unwrap();

                sz = read_u32(&self.head_buf[232..]).unwrap();
                sz -= removed_bytes;
                (&mut self.head_buf[232..236]).write_all(&sz.to_le_bytes()[..]).unwrap();

            }
        }
    }
}

impl RawFace {
    fn reverse(&mut self) {
        std::mem::swap(&mut self.v2, &mut self.v3);
    }
}

impl RawVertex {

    #[inline]
    fn scale(&mut self, factor: f64) {
        self.x = (self.x as f64 * factor) as f32;
        self.y = (self.y as f64 * factor) as f32;
        self.z = (self.z as f64 * factor) as f32;
    }

    #[inline]
    fn offset(&mut self, dx: f32, dy: f32, dz: f32) {
        self.x += dx;
        self.y += dy;
        self.z += dz;
    }

    #[inline]
    fn mirror_z(&mut self) {
        self.z = 0f32 - self.z;
    }
}

impl RawBBox {

    #[inline]
    fn scale(&mut self, factor: f64) {
        self.v_min.scale(factor); 
        self.v_max.scale(factor); 
    }

    #[inline]
    fn offset(&mut self, dx: f32, dy: f32, dz: f32) {
        self.v_min.offset(dx, dy, dz);
        self.v_max.offset(dx, dy, dz);
    }

    #[inline]
    fn mirror_z(&mut self) {
        let min_z = 0f32 - self.v_max.z;
        let max_z = 0f32 - self.v_min.z;
        self.v_min.z = min_z;
        self.v_max.z = max_z;
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

impl Eq for RawVertex {}

impl Eq for RawPoint {}

use std::hash::{Hash, Hasher};

impl Hash for RawVertex {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write(&self.x.to_le_bytes()[..]);
        h.write(&self.y.to_le_bytes()[..]);
        h.write(&self.z.to_le_bytes()[..]);
    }
}

impl Hash for RawPoint {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write(&self.x.to_le_bytes()[..]);
        h.write(&self.y.to_le_bytes()[..]);
    }
}
