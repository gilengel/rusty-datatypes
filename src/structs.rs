use proc_macro2::Span;
use syn::Ident;

use crate::types::{int::IntegerType, float::FloatType};

#[derive(Debug, Clone, PartialEq)]

pub(crate) enum DatatypeEndianness {
    LittleEndian,
    BigEndian
}

impl Default for DatatypeEndianness {
    fn default() -> Self {
        DatatypeEndianness::BigEndian
    }
}

impl From<&str> for DatatypeEndianness {
    fn from(value: &str) -> Self {
        match value {
            "LittleEndian" => DatatypeEndianness::LittleEndian,
            "BigEndian" => DatatypeEndianness::BigEndian,
            _ => panic!("Invalid DatatypeEndianness value")
        }
    }
}

impl TryFrom<Ident> for DatatypeEndianness {
    type Error = syn::Error;

    fn try_from(ident: syn::Ident) -> Result<Self, Self::Error> {
        let value = ident.to_string();
        let value = value.as_str();
        match value {
            "LittleEndian" => Ok(DatatypeEndianness::LittleEndian),
            "BigEndian" => Ok(DatatypeEndianness::BigEndian),
            _ => Err(syn::Error::new(ident.span(), "Invalid DatatypeEndianness value"))
        }
    }
}

impl<'a> Into<&'a str> for &DatatypeEndianness {
    fn into(self) -> &'a str {
        match self {
            DatatypeEndianness::LittleEndian => "LittleEndian",
            DatatypeEndianness::BigEndian => "BigEndian",
        }
    }
}



#[derive(Debug)]

pub(crate) enum DatatypeAttributeType {
    PrimitiveInteger(IntegerType),
    PrimitiveFloat(FloatType),
    Struct(String),
    String,
    // length type, position of length attribute (if None than the normal order will be used)
    Collection(String, Option<u8>),
    // the length of the collection as seperate attribute so that it can easily positioned arbitrarily
    CollectionLength(IntegerType),
}

#[derive(Debug)]
pub(crate) struct DatatypeAttribute {
    pub(crate) name: Ident,
    pub(crate) ty: DatatypeAttributeType,
    pub(crate) endianness: DatatypeEndianness,
    pub(crate) position: Option<u8>,
    pub(crate) position_span: Option<Span>,
    pub(crate) reserved: bool,
}