use std::fmt;

use super::{Nmf, Header, Name, Submaterial, Object};


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


impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Header::FromObj => write!(f, "fromObj"),
            Header::B3dmh10 => write!(f, "B3DMH 10")
        }
    }
}


impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Name::Valid(s, _) => write!(f, "\"{}\"", s),
            Name::Empty => write!(f, "Invalid (empty)"),
            Name::InvalidUtf8(_, e) => write!(f, "Invalid (utf-8 error: {})", e)
        }
    }
}


impl fmt::Display for Submaterial<'_> {
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
