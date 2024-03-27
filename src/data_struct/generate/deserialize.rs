use proc_macro2::{TokenStream as TokenStream2, Ident};

use crate::{structs::{DatatypeAttributeType, DatatypeAttribute}, types::{int::IntegerType, float::FloatType}};

fn quote_deserialize_primitive_integer(t: &IntegerType, name: &Ident, endianness: &TokenStream2) -> TokenStream2
{
    let ty : &str = t.clone().into();
    let read = format_ident!("read_{}", ty);

    match t{
        IntegerType::U8 | IntegerType::I8 => quote! { let #name = byte_stream.#read()?; },
        _ => quote! { let #name = byte_stream.#read::<byteorder::#endianness>()?; }
    }   
}

fn quote_deserialize_primitive_float(t: &FloatType, name: &Ident, endianness: &TokenStream2) -> TokenStream2
{
    let ty : &str = t.clone().into();
    let read = format_ident!("read_{}", ty);

    quote! { let #name = byte_stream.#read::<byteorder::#endianness>()?; }     
}

fn quote_deserialize_primitive_struct(ty: &String, name: &Ident) -> TokenStream2
{
    let ty: TokenStream2 = ty.parse().unwrap();

    quote! { let #name = #ty::deserialize(byte_stream)?; }          
}

fn quote_deserialize_primitive_collection(ty: &String, size: &Option<u8>, name: &Ident, attribute: &DatatypeAttribute) -> TokenStream2
{
    match size {
        Some(size) => quote! {
            let mut #name: [#ty; #size];
            for i in 0..#size {
                #name[i].push(#ty::deserialize(byte_stream)?);
            }
        },
        None =>  {
            let var_name = format_ident!("{}_len", attribute.name);
            let ty = format_ident!("{}", ty);

            quote! {

                let mut #name: Vec<#ty> = Vec::with_capacity(#var_name as usize);
                for _ in 0..#var_name {
                    #name.push(#ty::deserialize(byte_stream)?);
                }
            }
        }
    }       
}

fn quote_deserialize_primitive_collection_length(t: &IntegerType, attribute: &DatatypeAttribute, endianness: &TokenStream2) -> TokenStream2
{
    let var_name = format_ident!("{}_len", attribute.name);

    let ty : &str = t.clone().into();
    let read = format_ident!("read_{}", ty);

    match t{
        IntegerType::U8 | IntegerType::I8 => quote! { let #var_name = byte_stream.#read()?; },
        _ => quote! { let #var_name = byte_stream.#read::<byteorder::#endianness>()?; }
    }     
}
pub(crate) fn produce_deserialize_impl(
    name: &Ident, attrs: &Vec<DatatypeAttribute>,
) -> TokenStream2 {

    let attribute_names = attrs
    .iter()
    .filter(|attribute| attribute.reserved == false)
    .filter(|attribute| match attribute.ty {
        DatatypeAttributeType::PrimitiveInteger(_) |
        DatatypeAttributeType::PrimitiveFloat(_) |
        DatatypeAttributeType::Struct(_) |
        DatatypeAttributeType::String |
        DatatypeAttributeType::Collection(_, _) => true,
        DatatypeAttributeType::CollectionLength(_) => false,
    })
    .map(|attribute| {
        let name = format_ident!("{}", attribute.name);
        quote! { #name }
    });

    let deserialize_impl = attrs
    .iter()
    .map(|attribute| {
       
        let endianness: &str = (&attribute.endianness).into();
        let endianness: TokenStream2 = endianness.parse().unwrap();
        match &attribute.ty {
            DatatypeAttributeType::PrimitiveInteger(t) => quote_deserialize_primitive_integer(t, &attribute.name, &endianness),
            DatatypeAttributeType::PrimitiveFloat(t) => quote_deserialize_primitive_float(t, &attribute.name, &endianness),
            DatatypeAttributeType::Struct(ty) => quote_deserialize_primitive_struct(ty, &attribute.name),
            DatatypeAttributeType::String => quote! {},
            DatatypeAttributeType::Collection(ty, size) => quote_deserialize_primitive_collection(ty, size, &attribute.name, attribute),
            DatatypeAttributeType::CollectionLength(t) => quote_deserialize_primitive_collection_length(t, attribute, &endianness),
        }
    })
    .collect::<Vec<_>>();

    quote! {
        impl crate::datatypes::Deserialize for #name {
            fn deserialize(byte_stream: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self> where Self: Sized {
                use byteorder::{BigEndian, ReadBytesExt};

                #(#deserialize_impl)*
                
                Ok(#name {
                    #(#attribute_names),*                    
                })
            }
        }
    }
}
