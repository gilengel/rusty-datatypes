use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, Ident, Error};

use crate::structs::DatatypeEndianness;
use crate::types::int::IntegerType;

use super::structs::{DataEnumArg, DataEnumArgs};

impl Parse for DataEnumArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed_args: Punctuated<DataEnumArg, syn::Token![,]> =
            input.parse_terminated(DataEnumArg::parse, Token![,])?;

        let args: Vec<DataEnumArg> = parsed_args.into_iter().collect();

        Ok(DataEnumArgs { args })
    }
}

mod kw {
    syn::custom_keyword!(ty);
    syn::custom_keyword!(endianness);
}

pub(crate) fn parse_endianness(input: ParseStream) -> syn::Result<DataEnumArg> {
    input.parse::<kw::endianness>()?;
    input.parse::<Token![=]>()?;

    let endianness: Ident = input.parse()?;
    let endianness = DatatypeEndianness::try_from(endianness)?;
    Ok(DataEnumArg::Endianness(endianness))
}

fn parse_ty(input: ParseStream) -> syn::Result<DataEnumArg> {
    input.parse::<kw::ty>()?;
    input.parse::<Token![=]>()?;

    let ty: Ident = input.parse()?;
    let ty = IntegerType::try_from(ty)?;

    Ok(DataEnumArg::Type(ty))
}

impl Parse for DataEnumArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::endianness) {
            return parse_endianness(&input);
        }
    
        if lookahead.peek(kw::ty) {
            return parse_ty(&input);
        }
    
        Err(Error::new(input.span(), "Unknown attribute"))
    }
}
