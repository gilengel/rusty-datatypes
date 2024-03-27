use proc_macro2::{Ident, TokenStream as TokenStream2};


use crate::{
    structs::{DatatypeAttribute, DatatypeAttributeType},
    types::{float::FloatType, int::IntegerType},
};

fn quote_serialize_size_primitive_integer(t: &IntegerType) -> TokenStream2 {
    let size = t.size();

    quote! {#size}
}
fn quote_serialize_size_primitive_float(t: &FloatType) -> TokenStream2 {
    let size = t.size();

    quote! {#size}
}
fn quote_serialize_size_primitive_string(attribute_name: &Ident) -> TokenStream2 {
    quote! {self.#attribute_name.len() + 1}
}

fn quote_serialize_size_primitive_collection(
    embedded_ty: &String,
    size: &Option<u8>,
    attribute_name: &Ident,
) -> TokenStream2 {
    match IntegerType::try_from(embedded_ty.as_str()) {
        // for primitives like u8,u16 etc we simply need the length * bytes of the primitive
        Ok(embbed_type) => {
            let embedded_size = embbed_type.size();

            match size {
                Some(size) => quote! {
                    (#size as u32 * #embedded_size)  as u32
                },
                None => quote! {
                    (self.#attribute_name.len() as u32 * #embedded_size)  as u32
                },
            }
        }

        // if we failed to convert the type to a primitive we assume a custom type that must implement a size function
        // or in other words: the embedded type is also a Datatype using the macro.
        Err(_) => quote! {
            self.#attribute_name.iter().fold(0, |acc, embedded_type| acc + embedded_type.size())
        },
    }
}
fn quote_serialize_size_primitive_struct(attribute_name: &Ident) -> TokenStream2 {
    quote! {self.#attribute_name.size()}
}

fn quote_serialize_primitive_integer(
    attribute: &DatatypeAttribute,
    t: &IntegerType,
    endianness: &TokenStream2,
) -> TokenStream2 {
    let attribute_name = &attribute.name;

    let ty: Ident = t.into();
    let write = format_ident!("write_{}", ty);

    let writer_fragment = match t {
        IntegerType::U8 | IntegerType::I8 => quote! { writer.#write },
        _ => quote! { writer.#write::<byteorder::#endianness> },
    };

    match attribute.reserved {
        true => {
            let var_name = format_ident!("_{}", attribute_name);

            quote! { 
                let mut #var_name : #ty = 0; 
                #writer_fragment(#var_name)?;
            }
        }
        false => quote! { #writer_fragment(self.#attribute_name)?; },
    }
}
fn quote_serialize_primitive_float(
    t: &FloatType,
    name: &Ident,
    endianness: &TokenStream2,
) -> TokenStream2 {
    let ty: &str = t.clone().into();
    let write = format_ident!("write_{}", ty);

    quote! { writer.#write::<byteorder::#endianness>(self.#name)?; }
}
fn quote_serialize_primitive_string() -> TokenStream2 {
    quote! {
        writer.write_all(&self.raw_message.as_bytes())?;
        writer.write_u8(0)?;
    }
}

fn quote_serialize_primitive_collection(
    _: &String,
    name: &Ident,
    size: &Option<u8>,
    _: &DatatypeAttribute,
) -> TokenStream2 {
    // TODO: refactor this so that Vec<u8> etc types can be serialized as well.

    /*
    if let Ok(_) = IntegerType::try_from(ty) {
        //let ty : &str = t.clone().into();
        let write = format_ident!("write_{}", ty);

        emit_error!(Span::call_site(), "{:?}", attribute);
        //return quote !{
        //    writer.#write(#)
        //}
    }*/

    match size {
        Some(_) => quote! {
            for i in 0..#size {
                entry.serialize(writer)?;
            }
        },
        None => quote! {
            for entry in &self.#name {
                entry.serialize(writer)?;
            }
        },
    }
}

fn quote_serialize_primitive_collection_len(
    t: &IntegerType,
    name: &Ident,
    endianness: &TokenStream2,
) -> TokenStream2 {
    let ty: &str = t.clone().into();
    let write = format_ident!("write_{}", ty);

    let ty = format_ident!("{}", ty);
    match t {
        IntegerType::U8 | IntegerType::I8 => quote! { writer.#write(self.#name.len() as #ty)?; },
        _ => quote! {writer.#write::<byteorder::#endianness>(self.#name.len() as #ty)?; },
    }
}

pub(crate) fn produce_serialize_impl(name: &Ident, attrs: &Vec<DatatypeAttribute>) -> TokenStream2 {
    let size_impl = attrs
        .iter()
        .map(|attribute| {
            let attribute_name = format_ident!("{}", attribute.name);

            match &attribute.ty {
                DatatypeAttributeType::PrimitiveInteger(t) => {
                    quote_serialize_size_primitive_integer(t)
                }
                DatatypeAttributeType::PrimitiveFloat(t) => quote_serialize_size_primitive_float(t),
                DatatypeAttributeType::String => {
                    quote_serialize_size_primitive_string(&attribute_name)
                }
                DatatypeAttributeType::CollectionLength(t) => {
                    quote_serialize_size_primitive_integer(t)
                }
                DatatypeAttributeType::Collection(embedded_type, size) => {
                    quote_serialize_size_primitive_collection(embedded_type, size, &attribute_name)
                }
                DatatypeAttributeType::Struct(_) => {
                    quote_serialize_size_primitive_struct(&attribute_name)
                }
            }
        })
        .collect::<Vec<_>>();

    let serialize_impl = attrs
        .iter()
        .map(|attribute| {
            let endianness: &str = (&attribute.endianness).into();
            let endianness: proc_macro2::TokenStream = endianness.parse().unwrap();
            match &attribute.ty {
                DatatypeAttributeType::PrimitiveInteger(t) => {
                    quote_serialize_primitive_integer(attribute, t, &endianness)
                }
                DatatypeAttributeType::PrimitiveFloat(t) => {
                    quote_serialize_primitive_float(t, &attribute.name, &endianness)
                }
                DatatypeAttributeType::Struct(_) => {
                    let name = &attribute.name;
                    quote! {
                        self.#name.serialize(writer)?; 
                    }
                },
                DatatypeAttributeType::String => quote_serialize_primitive_string(),
                DatatypeAttributeType::Collection(ty, size) => {
                    quote_serialize_primitive_collection(ty, &attribute.name, size, attribute)
                }
                DatatypeAttributeType::CollectionLength(t) => {
                    quote_serialize_primitive_collection_len(t, &attribute.name, &endianness)
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl crate::datatypes::Serialize for #name {

            fn serialize(&self, writer: &mut std::io::BufWriter<std::fs::File>) -> std::io::Result<()> {
                use byteorder::{BigEndian, WriteBytesExt};

                #(#serialize_impl)*

                Ok(())
            }

            fn size(&self) -> u32 {
                #(#size_impl) +*
            }
        }
    }
}
