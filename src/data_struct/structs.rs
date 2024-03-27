use syn::{Ident, LitInt};

use crate::{structs::DatatypeEndianness, types::int::IntegerType};

#[derive(PartialEq, Debug, Clone)]

pub(crate) enum DataStructArg {
    FixedSize(LitInt)
}

pub(crate) struct DataStructArgs(pub(crate) Vec<DataStructArg>);

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum DataFieldArg {
    Endianness(DatatypeEndianness),
    LengthType(IntegerType),
    Position(LitInt),
    LengthPosition(LitInt),
    SerializeFunction(Ident),
    //DeserializeFunction(Ident),
    Ignore,
    Padding,
    //Coditional(Ident),
    //Version(Ident),
    Reserved,

}

