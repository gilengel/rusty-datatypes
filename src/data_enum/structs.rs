use crate::{structs::DatatypeEndianness, types::int::IntegerType};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum DataEnumArg {
    Endianness(DatatypeEndianness),
    Type(IntegerType)
}


#[derive(Debug)]
pub(crate) struct DataEnumArgs {
    pub args: Vec<DataEnumArg>,
}