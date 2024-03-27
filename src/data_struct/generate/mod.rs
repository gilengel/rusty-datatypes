use proc_macro2::TokenStream as TokenStream2;

use crate::{
    data_struct::{
        generate::{deserialize::produce_deserialize_impl, serialize::produce_serialize_impl},
        parse::DataField,
    },
    structs::DatatypeAttribute,
};

use super::parse::ItemStruct;

pub(crate) mod deserialize;
pub(crate) mod serialize;

pub(crate) fn produce(
    datatype_struct: &ItemStruct,
    attrs: &Vec<DatatypeAttribute>,
) -> TokenStream2 {
    let name = &datatype_struct.ident;

    let serialize_impl = produce_serialize_impl(&name, attrs);
    let deserialize_impl = produce_deserialize_impl(&name, attrs);

    let visibility = &datatype_struct.visibility;
    let attributes = &datatype_struct.attrs;

    let filtered: Vec<&DataField> = datatype_struct
        .fields
        .iter()
        .filter(|x| {
            x.attrs
                .iter()
                .filter(|x| match x {
                    crate::data_struct::structs::DataFieldArg::Reserved => true,
                    _ => false,
                })
                .count()
                == 0
        })
        .collect();

    let ast = quote! {
        #(#attributes),*
        #visibility struct #name {
            #(#filtered),*
        }
    };

    //abort!(Span::call_site(), "{:#?}", filtered);
    quote! {
        #ast

        #serialize_impl

        #deserialize_impl
    }
}
