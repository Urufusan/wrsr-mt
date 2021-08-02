use super::{NmfSlice, ObjectSlice};


#[derive(Debug)]
pub enum Modifier {
    RemoveObject(String),
    Scale(f64),
}


#[derive(Debug)]
pub enum ModifyError {
    CannotRemoveObject(String),
}


impl<'a> NmfSlice<'a> {

    pub fn write_with_modifiers<W: std::io::Write>(&self, mods: &mut Vec<Modifier>, writer: W) -> Result<(), ModifyError> {
        todo!()
    }
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
//----------------------------------------------------------------
