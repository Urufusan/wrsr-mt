use std::fmt;
use std::fs;
use std::path::Path;
use std::io::{self, Read, Seek, Write};
use std::convert::TryInto;

pub mod object_full;

pub use object_full::ObjectFull;


#[derive(Debug)]
pub enum Error {
    FileIO(io::Error),
    HeaderEOF(ChopEOF),
    UnknownNmfType,
    FileLengthMismatch(usize, u64),
    Submaterial(usize, io::Error),
    Object(usize, ObjectError),
    U32Conversion(std::num::TryFromIntError),
    WriteObject(usize, io::Error)
}


#[derive(Debug)]
pub enum ObjectError {
    FileIO(io::Error),
    SliceReadU32,
    WrongIndicesCount(u32),
    ZeroSubmaterials,
    Allocation(String),
}


#[derive(Debug)]
pub struct ChopEOF {
    need: usize,
    have: usize
}

//--------------------------------


pub struct NmfBuf<T> {
    nmf_type: NmfType,
    submaterials: Vec<NameBuf>,
    pub objects: Vec<T>,
    remainder: u64
}


pub enum NmfType {
    FromObj,
    B3dmh10
}


pub struct NameBuf {
    bytes: [u8; 64],
    displayed: usize
}


pub struct ObjectInfo {
    name: NameBuf,
    range: std::ops::Range<u64>,
    vertices: u32,
    faces: u32,
    submat_main: u32,
    submat_rest: Vec<u32>
}


pub type NmfInfo = NmfBuf<ObjectInfo>;
pub type NmfBufFull = NmfBuf<ObjectFull>;


pub trait ObjectReader<R: Read> {
    fn from_reader(rdr: &mut R) -> Result<Self, ObjectError> where Self: Sized;
}


//----------------------------------------------------------------------------------


impl<T: ObjectReader<fs::File>> NmfBuf<T> {

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<NmfBuf<T>, Error> {
        let path: &Path = path.as_ref();
        
        let mut buf = [0; 512];

        let mut file = fs::File::open(path).map_err(Error::FileIO)?;
        let file_len = file.metadata().map_err(Error::FileIO).map(|md| md.len())?;

        let (nmf_type, submat_count, obj_count, nmf_len) = {
            let slice = &mut buf[0 .. 20];
            file.read_exact(slice).map_err(Error::FileIO)?;
            let mut chop = SliceChopper::from(slice);
            
            let nmf_type  = chop.chop_subslice(8).map_err(Error::HeaderEOF).and_then(|s| NmfType::from_slice(s).ok_or(Error::UnknownNmfType))?;
            let sm_count  = chop.chop_u32size().map_err(Error::HeaderEOF)?;
            let obj_count = chop.chop_u32size().map_err(Error::HeaderEOF)?;
            let nmf_len   = chop.chop_u32size().map_err(Error::HeaderEOF)?;

            (nmf_type, sm_count, obj_count, nmf_len)
        };

        if nmf_len as u64 != file_len {
            return Err(Error::FileLengthMismatch(nmf_len, file_len));
        }

        let mut submaterials = Vec::<NameBuf>::with_capacity(submat_count);
        for i in 0 .. submat_count {
            submaterials.push(NameBuf::from_reader(&mut file).map_err(|e| Error::Submaterial(i, e))?);
        }
        
        let mut objects = Vec::<T>::with_capacity(obj_count);
        for i in 0 .. obj_count {
            objects.push(T::from_reader(&mut file).map_err(|e| Error::Object(i, e))?);
        }

        let remainder = file_len - file.stream_position().map_err(Error::FileIO)?;

        Ok(NmfBuf { nmf_type, submaterials, objects, remainder })
    }
}


impl<R: Read + Seek> ObjectReader<R> for ObjectInfo {
    fn from_reader(rdr: &mut R) -> Result<ObjectInfo, ObjectError> {

        #[inline]
        fn skip<R: Seek>(reader: &mut R, n: u32) -> Result<u64, ObjectError> {
            reader.seek(io::SeekFrom::Current(n as i64)).map_err(ObjectError::FileIO)
        }

        #[inline]
        fn read_u32<R: Read>(reader: &mut R) -> Result<u32, ObjectError> {
            let mut b4 = [0u8; 4];
            reader.read_exact(&mut b4[..]).map_err(ObjectError::FileIO)?;
            Ok(u32::from_le_bytes(b4))
        }

        let start = rdr.stream_position().map_err(ObjectError::FileIO)?;

        skip(rdr, 8)?;
        let name = NameBuf::from_reader(rdr).map_err(ObjectError::FileIO)?;
        skip(rdr, 164)?;

        let vertices = read_u32(rdr)?;
        let indices = read_u32(rdr)?;

        let submats = read_u32(rdr)?;
        if submats == 0 {
            return Err(ObjectError::ZeroSubmaterials)
        }

        let mut submat_rest = Vec::with_capacity(submats as usize - 1);

        let faces = get_faces_count(indices)?;
        let skip_len = indices_len_bytes(indices) + geometry_len_bytes(vertices, faces);
        // 12 (pre-indices magic bytes) + 8 (primary material indices)
        skip(rdr, 20 + skip_len)?;

        let submat_main = read_u32(rdr)?;

        for _ in 1 .. submats {
            skip(rdr, 8)?;
            submat_rest.push(read_u32(rdr)?);
        }

        let end = rdr.stream_position().map_err(ObjectError::FileIO)?;

        Ok(ObjectInfo { 
            name, 
            range: start .. end,
            vertices,
            faces,
            submat_main,
            submat_rest
        })
    }
}


impl NmfBuf<ObjectFull> {

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let path: &Path = path.as_ref();
        
        let f_out = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)
                        .map_err(Error::FileIO)?;

        let mut wr = io::BufWriter::new(f_out);

        self.nmf_type.write_bytes(&mut wr).map_err(Error::FileIO)?;
        write_num_u32(self.submaterials.len(), &mut wr)?;
        write_num_u32(self.objects.len(), &mut wr)?;
        write_num_u32(0, &mut wr)?;

        for sm in self.submaterials.iter() {
            wr.write_all(&sm.bytes).map_err(Error::FileIO)?;
        }

        for (i, o) in self.objects.iter().enumerate() {
            o.write_bytes(&mut wr).map_err(|e| Error::WriteObject(i, e))?;
        }

        let len = wr.stream_position().map_err(Error::FileIO)?;
        wr.seek(io::SeekFrom::Start(16)).map_err(Error::FileIO)?;
        write_num_u32(len, &mut wr)?;

        wr.flush().map_err(Error::FileIO)
    }
}


impl NmfType {
    const FROM_OBJ: &'static [u8] = b"fromObj\0";
    const B3DMH_10: &'static [u8] = b"B3DMH\010";

    fn from_slice(bytes: &[u8]) -> Option<NmfType> {
        match bytes {
            Self::FROM_OBJ => Some(NmfType::FromObj),
            Self::B3DMH_10 => Some(NmfType::B3dmh10),
            _ => None
        }
    }

    fn write_bytes<W: Write>(&self, mut wr: W) -> Result<(), io::Error> {
        let slice = match self {
            NmfType::FromObj => Self::FROM_OBJ,
            NmfType::B3dmh10 => Self::B3DMH_10,
        };

        wr.write_all(slice)
    }
}


impl NameBuf {

    const BUF_LENGTH: usize = 64;

    fn from_reader<R: Read>(rdr: &mut R) -> Result<NameBuf, io::Error> {
        let mut name = NameBuf {
            bytes: [0; Self::BUF_LENGTH],
            displayed: 0
        };

        rdr.read_exact(&mut name.bytes[..])?;
        name.displayed = Self::get_len(&name.bytes[..]);
        Ok(name)
    }

    fn as_str<'a>(&'a self) -> &'a str {
        if self.displayed > 0 {
            let s = unsafe { std::str::from_utf8_unchecked(self.bytes.get_unchecked(0 .. self.displayed)) };
            &s
        } else {
            &"<not displayable>"
        }
    }

    fn get_len(bytes: &[u8]) -> usize {
        let len = bytes.iter().position(|&x| x == 0).unwrap_or(bytes.len());

        if len > 0 {
            let s = unsafe { bytes.get_unchecked(0 .. len) };
            if std::str::from_utf8(s).is_ok() {
                len
            } else { 0 }
        } else { 0 }
    }
}


#[inline]
const fn indices_len_bytes(indices: u32) -> u32 {
    indices * 2
}

#[inline]
const fn geometry_len_words(vertices: u32, faces: u32) -> u32 {
    vertices * (3 + 9 + 2) + faces * (4 + 6)
}

#[inline]
const fn geometry_len_bytes(vertices: u32, faces: u32) -> u32 {
    geometry_len_words(vertices, faces) * 4
}

#[inline]
fn get_faces_count(indices: u32) -> Result<u32, ObjectError> {
    let (c, rm) = num::integer::div_rem(indices, 3);
    if rm != 0 {
        return Err(ObjectError::WrongIndicesCount(indices));
    }

    Ok(c)
}

#[inline]
fn write_num_u32<T: Write, N: TryInto<u32, Error = std::num::TryFromIntError>>(i: N, wr: &mut T) -> Result<(), Error> {
    let i = i.try_into().map_err(Error::U32Conversion)?;
    wr.write_all(&i.to_le_bytes()).map_err(Error::FileIO)
}

//-----------------------------------------------------------------------------


struct SliceChopper<'a> {
    slice: &'a [u8],
}


impl<'a> From<&'a mut [u8]> for SliceChopper<'a> {
    fn from(slice: &'a mut [u8]) -> SliceChopper<'a> {
        SliceChopper { slice }
    }
}


impl<'a> SliceChopper<'a> {
    fn chop_subslice(&mut self, len: usize) -> Result<&'a [u8], ChopEOF> {
        if len > self.slice.len() {
            Err(ChopEOF { need: len, have: self.slice.len() })
        } else {
            let (r, s) = self.slice.split_at(len);
            self.slice = s;
            Ok(r)
        }
    }

    fn chop_u32(&mut self) -> Result<u32, ChopEOF> {
        let s = self.chop_subslice(std::mem::size_of::<u32>())?;
        Ok(u32::from_le_bytes(s.try_into().unwrap()))
    }

    fn chop_u32size(&mut self) -> Result<usize, ChopEOF> {
        self.chop_u32().map(|x| x as usize)
    }
}

//-----------------------------------------------------------------------------


impl fmt::Display for NmfBuf<ObjectInfo> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        #[inline]
        const fn count_digits(mut x: u32) -> usize {
            let mut width = 1_usize;
            while x > 10 {
                x = x / 10;
                width += 1;
            }

            width
        }


        writeln!(f, "Type: {}", self.nmf_type)?;

        writeln!(f, "Submaterials: {}", self.submaterials.len())?;
        for (i, sm) in self.submaterials.iter().enumerate() {
            writeln!(f, "{:2}) {}", i, sm)?;
        }

        writeln!(f, "Objects: {}", self.objects.len())?;

        const H_NAME: &str = "NAME";
        const H_VERTS: &str = "VERTS";
        const H_FACES: &str = "FACES";

        let mut w_name = H_NAME.len();

        let mut w_verts = H_VERTS.len();
        let mut max_verts = 10_u32.pow(w_verts as u32) - 1;

        let mut w_faces = H_FACES.len();
        let mut max_faces = 10_u32.pow(w_faces as u32) - 1;

        for o in self.objects.iter() {
            w_name = std::cmp::max(w_name, o.name.as_str().chars().count());
            if o.vertices > max_verts {
                max_verts = o.vertices;
                w_verts = count_digits(o.vertices);
            }

            if o.faces > max_faces {
                max_faces = o.faces;
                w_faces = count_digits(o.faces);
            }
        }

        writeln!(f, "        LOCATION      {0:^1$}  {2:^3$}  {4:^5$}  SUBMATERIALS", H_NAME, w_name, H_VERTS, w_verts, H_FACES, w_faces)?;
        for (i, o) in self.objects.iter().enumerate() {
            write!(f, "{:2}) [{:0>6x}..{:0>6x}]  ", i, o.range.start, o.range.end)?;
            write!(f, "{0:<1$}  ", o.name, w_name)?;
            write!(f, "{0:>1$}  ", o.vertices, w_verts)?;
            write!(f, "{0:>1$}  ", o.faces, w_faces)?;

            write!(f, "[{}", o.submat_main)?;
            for smp in o.submat_rest.iter() {
                write!(f, ", {}", smp)?;
            }
            
            writeln!(f, "]")?;
        }

        if self.remainder > 0 {
            writeln!(f, "WARNING: Nmf parsed with leftover bytes ({})", self.remainder)?;
        }

        Ok(())
    }
}


impl fmt::Display for NmfType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NmfType::FromObj => write!(f, "fromObj"),
            NmfType::B3dmh10 => write!(f, "B3DMH 10")
        }
    }
}


impl fmt::Display for NameBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.pad(self.as_str())
    }
}


impl fmt::Display for NmfBuf<ObjectFull> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        writeln!(f, "Type: {}", self.nmf_type)?;

        writeln!(f, "Submaterials: {}", self.submaterials.len())?;
        for (i, sm) in self.submaterials.iter().enumerate() {
            writeln!(f, "{:2}) {}", i, sm)?;
        }

        writeln!(f, "Objects: {}", self.objects.len())?;
        for (i, o) in self.objects.iter().enumerate() {
            writeln!(f, "{:2}) {}", i, o.name())?;
        }

        if self.remainder > 0 {
            writeln!(f, "WARNING: Nmf parsed with leftover bytes ({})", self.remainder)?;
        }

        Ok(())
    }
}
