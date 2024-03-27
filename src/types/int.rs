use quote::ToTokens;
use syn::{Ident, Type, spanned::Spanned};

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum IntegerType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128
}


impl Default for IntegerType {
    fn default() -> Self {
        IntegerType::U16
    }
}

impl TryFrom<Ident> for IntegerType {
    type Error = syn::Error;

    fn try_from(ident: proc_macro2::Ident) -> Result<Self, Self::Error> {
        let value = ident.to_string();
        let value = value.as_str();
        match value {
            "u8"  => Ok(IntegerType::U8),
            "i8"  => Ok(IntegerType::I8),
            "u16"  => Ok(IntegerType::U16),
            "i16"  => Ok(IntegerType::I16),
            "u32"  => Ok(IntegerType::U32),
            "i32"   => Ok(IntegerType::I32),
            "u64"  => Ok(IntegerType::U64),
            "i64"   => Ok(IntegerType::I64),
            "u128"  => Ok(IntegerType::U128),
            "i128 " => Ok(IntegerType::I128),
            _ => Err(syn::Error::new(ident.span(), "Invalid CollectionLenghType value"))
        }
    }
}

impl Into<Ident> for &IntegerType {
    fn into(self) -> Ident {
        Ident::new(self.into(), proc_macro2::Span::call_site())
    }
}

impl Into<proc_macro::Ident> for &IntegerType {
    fn into(self) -> proc_macro::Ident {
        proc_macro::Ident::new(self.into(), proc_macro::Span::call_site())
    }
}

impl TryFrom<Type> for IntegerType {
    type Error = syn::Error;

    fn try_from(ty: Type) -> Result<Self, Self::Error> {
        IntegerType::try_from(&ty)
    }
}

impl TryFrom<&Type> for IntegerType {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(x) => {
                match IntegerType::try_from(x.to_token_stream().to_string().as_str())                 {
                    Ok(x) => Ok(x),
                    Err(msg) => Err(syn::Error::new(ty.span(), msg)),
                }
            },
            _ => Err(syn::Error::new(ty.span(), "Invalid CollectionLenghType value"))
        }
    }
}

impl TryFrom<String> for IntegerType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        IntegerType::try_from(&value)
    }
}

impl TryFrom<&String> for IntegerType {
    type Error = &'static str;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        IntegerType::try_from(value.as_str())
    }
}

impl TryFrom<&str> for IntegerType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "u8"  => Ok(IntegerType::U8),
            "i8"  => Ok(IntegerType::I8),
            "u16"  => Ok(IntegerType::U16),
            "i16"  => Ok(IntegerType::I16),
            "u32"  => Ok(IntegerType::U32),
            "i32"   => Ok(IntegerType::I32),
            "u64"  => Ok(IntegerType::U64),
            "i64"   => Ok(IntegerType::I64),
            "u128"  => Ok(IntegerType::U128),
            "i128 " => Ok(IntegerType::I128),
            _ => Err("Invalid CollectionLenghType value")
        }
    }
}

impl<'a> Into<&'a str> for &IntegerType {
    fn into(self) -> &'a str {
        match self {
            IntegerType::U8 => "u8",
            IntegerType::U16 => "u16",
            IntegerType::U32 => "u32",
            IntegerType::U64 => "u64",
            IntegerType::U128 => "u128",
            IntegerType::I8 => "i8",
            IntegerType::I16 => "i16",
            IntegerType::I32 => "i32",
            IntegerType::I64 => "i64",
            IntegerType::I128 => "i128",
        }
    }
}

impl<'a> Into<&'a str> for IntegerType {
    fn into(self) -> &'a str {
        (&self).into()
    }
}

impl IntegerType {
    pub(crate) fn size(&self) -> u32 {
        match self {
            IntegerType::U8 => 1,
            IntegerType::U16 => 2,
            IntegerType::U32 => 4,
            IntegerType::U64 => 8,
            IntegerType::U128 => 16,
            IntegerType::I8 => 1,
            IntegerType::I16 => 2,
            IntegerType::I32 => 4,
            IntegerType::I64 => 8,
            IntegerType::I128 => 16,
        }
    }
}