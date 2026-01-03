use std::ffi::CString;
use bstr::{BStr, BString, ByteSlice};

// Convertible PETSCII string type
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PetString(BString);

impl PetString {
    pub fn new(b: &BString) -> PetString {
        PetString(b.clone())
    }
    fn asc2pet(a: u8) -> u8 {
        match a {
            0x41..=0x5A => a+0x80,
            0x61..=0x7A => a-0x20,
            0x7B..=0x7F => a+0x60,
            _ => a
        }
    }
    fn to_pet(a: &str) -> BString {
        let mut result = BString::new(vec![]);
    
        for c in a.chars() {
            result.push(Self::asc2pet(c as u8));
        }
        result
    }
    fn pet2asc(p: u8) -> u8 {
        match p as char {
            'a'..='z' => p-0x20,
            'A'..='Z' => p+0x20,
            'Á'..='Ú' => p-0x80,
            'Þ' => p-0x60,
            _ => p
        }
    }
    fn from_pet(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
    
        for c in self.0.as_slice() {
            result.push(Self::pet2asc(*c));
        }
        result
    }
    pub fn as_bstr(&self) -> &BStr {
        self.0.as_bstr()
    }
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}
impl From<String> for PetString {
    fn from(value: String) -> Self {
        PetString(Self::to_pet(&value))
    }
}
impl From<&str> for PetString {
    fn from(value: &str) -> Self {
        PetString(Self::to_pet(value))
    }
}
impl From<PetString> for String {
    fn from(value: PetString) -> String {
        match String::from_utf8(value.from_pet()) {
            Ok(s) => s,
            Err(e) => {
                let l = e.utf8_error().valid_up_to();
                let mut p = value.from_pet();
                p.truncate(l);
                String::from_utf8(p).unwrap()
            }
        }
    }
}
impl From<PetString> for BString {
    fn from(value: PetString) -> BString {
        value.0.to_owned()
    }
}
impl From<PetString> for CString {
    fn from(value: PetString) -> CString {
        CString::new(value.0.as_slice()).unwrap()
    }
}

pub fn _padded(s: &[u8], width: usize) -> BString {
    let mut pad = BString::new(s.to_vec());
    while pad.len()<width {
        pad.push(b' ');
    }
    pad
}
