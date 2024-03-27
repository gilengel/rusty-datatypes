use quote::ToTokens;
use syn::{Type, spanned::Spanned};

#[derive(Debug, Clone)]
pub(crate) enum FloatType {
    F32,
    F64
}

impl Default for FloatType {
    fn default() -> Self {
        FloatType::F32
    }
}

impl TryFrom<Type> for FloatType {
    type Error = syn::Error;

    fn try_from(ty: Type) -> Result<Self, Self::Error> {
        FloatType::try_from(&ty)
    }
}

impl TryFrom<&Type> for FloatType {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(x) => {
                match FloatType::try_from(x.to_token_stream().to_string().as_str())                 {
                    Ok(x) => Ok(x),
                    Err(msg) => Err(syn::Error::new(ty.span(), msg)),
                }
            },
            _ => Err(syn::Error::new(ty.span(), "Invalid CollectionLenghType value"))
        }
    }
}

impl TryFrom<&str> for FloatType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "f32"  => Ok(FloatType::F32),
            "f64"  => Ok(FloatType::F64),

            _ => Err("Invalid FloatType value")
        }
    }
}

impl<'a> Into<&'a str> for FloatType {
    fn into(self) -> &'a str {
        match self {
            FloatType::F32 => "f32",
            FloatType::F64 => "f64",
        }
    }
}

impl FloatType {
    pub(crate) fn size(&self) -> u32 {
        match self {
            FloatType::F32 => 4,
            FloatType::F64 => 8,
        }
    }
}