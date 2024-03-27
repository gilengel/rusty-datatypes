use proc_macro2::Span;
use proc_macro_error::emit_warning;
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::Attribute;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use crate::parse::{get_collection_embedded_type, is_collection_type};
use crate::structs::{DatatypeAttribute, DatatypeAttributeType, DatatypeEndianness};
use crate::types::float::FloatType;
use crate::types::int::IntegerType;
use syn::token::Colon;
use syn::{
    braced, bracketed, parenthesized, token, Error, Ident, LitInt, Result, Token, Type, Visibility,
};

use super::structs::{DataFieldArg, DataStructArg, DataStructArgs};

mod kw {
    syn::custom_keyword!(field);
    syn::custom_keyword!(endianness);
    syn::custom_keyword!(length_ty);
    syn::custom_keyword!(reserved);
    syn::custom_keyword!(position);
    syn::custom_keyword!(length_position);
    syn::custom_keyword!(serialize);
    syn::custom_keyword!(deserialize);
    syn::custom_keyword!(ignore);
    syn::custom_keyword!(conditional);
    syn::custom_keyword!(version);
    syn::custom_keyword!(padding);

    pub(crate) mod st {
        syn::custom_keyword!(fixed_size);
    }
}



#[derive(Debug)]
pub struct ItemStruct {
    pub visibility: Visibility,
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<DataField, Token![,]>,
}

#[derive(Debug)]
pub(crate) struct DataField {
    pub(crate) visibility: Visibility,
    pub(crate) name: Ident,
    pub(crate) attrs: Vec<DataFieldArg>,
    pub(crate) colon: Token![:],
    pub(crate) ty: Type,
}

impl ToTokens for DataField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.visibility.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

fn get_endianness(attrs: &Vec<DataFieldArg>) -> DatatypeEndianness {
    attrs
        .iter()
        .find_map(|x| match x {
            DataFieldArg::Endianness(x) => Some(x.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn get_reserved(attrs: &Vec<DataFieldArg>) -> bool {
    attrs
    .iter()
    .find(|x| match x {
        DataFieldArg::Reserved => true,
        _ => false,
    })
    .is_some()
}

fn get_position(attrs: &Vec<DataFieldArg>) -> (Option<u8>, Option<Span>)
{
    attrs
    .iter()
    .find_map(|x| match x {
        DataFieldArg::Position(x) => {
            Some((Some(x.base10_parse::<u8>().unwrap()), Some(x.span())))
        }
        _ => None,
    })
    .unwrap_or((None, None))
}

fn get_length_position(attrs: &Vec<DataFieldArg>) -> (Option<u8>, Option<Span>)
{
    attrs
    .iter()
    .find_map(|x| match x {
        DataFieldArg::LengthPosition(x) => {
            Some((Some(x.base10_parse::<u8>().unwrap()), Some(x.span())))
        }
        _ => None,
    })
    .unwrap_or((None, None))
}

fn get_integer_ty(attrs: &Vec<DataFieldArg>) -> IntegerType {
    attrs
    .iter()
    .find_map(|x| match x {
        DataFieldArg::LengthType(x) => Some(x.clone()),
        _ => None,
    })
    .unwrap_or_default()
}

impl Into<Vec<DatatypeAttribute>> for &DataField {
    fn into(self) -> Vec<DatatypeAttribute> {
        let endianness = get_endianness(&self.attrs);
        let reserved = get_reserved(&self.attrs);
        let (position, position_span) = get_position(&self.attrs);

        if is_collection_type(&self.ty).is_some() {
            let length_ty = get_integer_ty(&self.attrs);
            let (length_position, length_position_span) = get_length_position(&self.attrs);
            let (collection_ty, collection_length) = get_collection_embedded_type(&self.ty);

            return vec![
                DatatypeAttribute {
                    name: self.name.clone(),
                    ty: DatatypeAttributeType::CollectionLength(length_ty),
                    endianness: endianness.clone(),
                    position: length_position,
                    position_span: length_position_span,
                    reserved,
                },
                DatatypeAttribute {
                    name: self.name.clone(),
                    ty: DatatypeAttributeType::Collection(collection_ty, collection_length),
                    endianness,
                    position,
                    position_span,
                    reserved,
                },
            ];
        }
        
        // TODO avoid the whole conversions + string conversion and use a better approach
        if self.ty.to_token_stream().to_string().as_str() == "String"
        {
            return vec![DatatypeAttribute {
                name: self.name.clone(),
                ty: DatatypeAttributeType::String,
                endianness,
                position,
                position_span,
                reserved,
            }];
        }

        if let Ok(x) = IntegerType::try_from(&self.ty) {
            return vec![DatatypeAttribute {
                name: self.name.clone(),
                ty: DatatypeAttributeType::PrimitiveInteger(x),
                endianness,
                position,
                position_span,
                reserved,
            }];
        }

        if let Ok(x) = FloatType::try_from(&self.ty) {
            return vec![DatatypeAttribute {
                name: self.name.clone(),
                ty: DatatypeAttributeType::PrimitiveFloat(x),
                endianness,
                position,
                position_span,
                reserved,
            }];
        }

        vec![DatatypeAttribute {
            name: self.name.clone(),
            ty: DatatypeAttributeType::Struct(self.ty.to_token_stream().to_string()),
            endianness,
            position,
            position_span,
            reserved,
        }]
    }
}

fn parse_endianness(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::endianness>()?;
    input.parse::<Token![=]>()?;

    let endianness: Ident = input.parse()?;
    let endianness = DatatypeEndianness::try_from(endianness)?;

    Ok(DataFieldArg::Endianness(endianness))
}

fn parse_length_ty(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::length_ty>()?;
    input.parse::<Token![=]>()?;

    let ty: Ident = input.parse()?;
    let ty = IntegerType::try_from(ty)?;

    Ok(DataFieldArg::LengthType(ty))
}
fn parse_length_position(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::length_position>()?;
    input.parse::<Token![=]>()?;

    let value = input.parse::<LitInt>()?;

    Ok(DataFieldArg::LengthPosition(value))
}

fn parse_position(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::position>()?;
    input.parse::<Token![=]>()?;
    let value = input.parse::<LitInt>()?;

    Ok(DataFieldArg::Position(value))
}

fn parse_serialize(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::serialize>()?;
    input.parse::<Token![=]>()?;
    let func = input.parse::<Ident>()?;

    Ok(DataFieldArg::SerializeFunction(func))
}

fn parse_deserialize(input: ParseStream) -> Result<DataFieldArg> {
    input.parse::<kw::deserialize>()?;
    input.parse::<Token![=]>()?;
    let func = input.parse::<Ident>()?;

    Ok(DataFieldArg::SerializeFunction(func))
}

fn parse_muu(input: ParseStream) -> Result<DataFieldArg> {
    let lookahead = input.lookahead1();
    if lookahead.peek(kw::endianness) {
        return parse_endianness(&input);
    }

    if lookahead.peek(kw::length_ty) {
        return parse_length_ty(&input);
    }

    if lookahead.peek(kw::reserved) {
        input.parse::<kw::reserved>()?;
        return Ok(DataFieldArg::Reserved);
    }
    if lookahead.peek(kw::position) {
        return parse_position(&input);
    }

    if lookahead.peek(kw::length_position) {
        return parse_length_position(&input);
    }

    if lookahead.peek(kw::serialize) {
        return parse_serialize(&input);
    }

    if lookahead.peek(kw::deserialize) {
        return parse_deserialize(&input);
    }

    if lookahead.peek(kw::ignore) {
        input.parse::<kw::ignore>()?;
        return Ok(DataFieldArg::Ignore);
    }

    if lookahead.peek(kw::conditional) {
        let conditional = input.parse::<kw::conditional>()?;
        input.parse::<Token![=]>()?;
        emit_warning!(
            conditional,
            "{}",
            "'conditional' is currently not implemented and will be ignored"
        );
    }

    if lookahead.peek(kw::version) {
        let version = input.parse::<kw::version>()?;
        emit_warning!(
            version,
            "{}",
            "'version' is currently not implemented and will be ignored"
        );
    }

    if lookahead.peek(kw::padding) {
        input.parse::<kw::padding>()?;
        return Ok(DataFieldArg::Padding)
    }

    Err(Error::new(input.span(), "Unknown attribute"))
}

pub(crate) fn single_parse_outer(input: ParseStream) -> Result<Vec<DataFieldArg>> {
    let mut content;

    let _pount_token = input.parse::<Token![#]>()?;
    let _bracket_token = bracketed!(content in input);

    content.parse::<kw::field>()?;

    let _bracket_token = parenthesized!(content in content);

    let args = content.parse_terminated(parse_muu, Token![,])?;
    let args: Vec<DataFieldArg> = args.iter().map(|x| (*x).clone()).collect();
    Ok(args)
}
pub fn parse_outer(input: ParseStream) -> Result<Vec<DataFieldArg>> {
    let mut attrs = Vec::new();
    while input.peek(Token![#]) {
        attrs.push(input.call(single_parse_outer)?);
    }

    let attrs = attrs
        .iter()
        .flatten()
        .map(|x| (*x).clone())
        .collect::<Vec<_>>();
    Ok(attrs)
}

fn parse_named(input: ParseStream) -> Result<DataField> {
    let attrs = input.call(parse_outer)?;
    let visibility = input.parse::<Visibility>()?;

    let name = if input.peek(Token![_]) {
        input.call(Ident::parse_any)
    } else {
        input.parse()
    }?;

    let colon = input.parse::<Colon>()?;
    let ty: Type = input.parse()?;

    Ok(DataField {
        attrs,
        visibility,
        name,
        colon,
        ty,
    })
}

impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let attrs = input.call(Attribute::parse_outer)?;

        let visibility = input.parse::<Visibility>()?;
        let _struct_token: Token![struct] = input.parse()?;

        Ok(ItemStruct {
            visibility,
            attrs,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(parse_named, Token![,])?,
        })
    }
}

impl Parse for DataStructArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::st::fixed_size) {
            return parse_endianness(&input);
        }

        Err(Error::new(input.span(), "Unknown attribute"))
    }
}

impl Parse for DataStructArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed_args: Punctuated<DataStructArg, syn::Token![,]> =
            input.parse_terminated(DataStructArg::parse, Token![,])?;

        let args: Vec<DataStructArg> = parsed_args.into_iter().collect();

        Ok(DataStructArgs(args))
    }
}