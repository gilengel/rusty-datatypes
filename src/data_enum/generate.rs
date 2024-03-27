use proc_macro2::{TokenStream as TokenStream2, Ident};
use syn::DeriveInput;


use crate::{structs::DatatypeEndianness, types::int::IntegerType};

use super::structs::{DataEnumArgs, DataEnumArg};


pub(crate) fn produce(ast: &DeriveInput, args: &DataEnumArgs) -> TokenStream2 {
    let endianness = match args.args.iter().find_map(|x| match x {
        DataEnumArg::Endianness(endianness) => Some(endianness),
    _ => None
    })
    {
        Some(endianness) => endianness.clone(),
        None => DatatypeEndianness::default(),
    };

    let endianness: &str = (&endianness).into();
    let endianness: proc_macro2::TokenStream = endianness.parse().unwrap();

    let ty = match args.args.iter().find_map(|x| match x {
        DataEnumArg::Type(ty) => Some(ty),
    _ => None
    })
    {
        Some(x) => x.clone(),
        None => IntegerType::U8,
    };
        
    
    let name = &ast.ident;

    let size = ty.size();  
    let ty_ident: Ident = (&ty).into(); 

    let from = format_ident!("from_{}", ty_ident);
    let to = format_ident!("to_{}", ty_ident);

    let write = format_ident!("write_{}", ty_ident);

    let write = match ty{
        IntegerType::U8 | IntegerType::I8 => quote! { writer.#write(num_traits::ToPrimitive::#to(self).unwrap())?;
        },
        _ => quote! { writer.#write::<#endianness>(num_traits::ToPrimitive::#to(self).unwrap())?; }
    }; 

    let read =format_ident!("read_{}", ty_ident);
    let read = match ty{
        
        IntegerType::U8 | IntegerType::I8 => quote! { num_traits::FromPrimitive::#from(byte_stream.#read()?) },
        _ => quote! {                                 num_traits::FromPrimitive::#from(byte_stream.#read::<#endianness>()?) }
    };     

    quote! {
        
        #[derive(Debug, PartialEq, Clone, Copy, num_derive::FromPrimitive, num_derive::ToPrimitive)]
        #[repr(#ty_ident)]        
        #ast
            
        
        impl crate::datatypes::Deserialize for #name {
            fn deserialize(byte_stream: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self>
            where
                Self: Sized,
            {
                use byteorder::{BigEndian, ReadBytesExt};
                let value = #read.unwrap();

                Ok(value)
            }
        }

        impl crate::datatypes::Serialize for #name {
            fn serialize(&self, writer: &mut std::io::BufWriter<std::fs::File>) -> std::io::Result<()>
            {
                use byteorder::{BigEndian, WriteBytesExt};
                #write

                Ok(())
            }
    
            fn size(&self) -> u32
            {
                #size
            }           
        }
        
        
    }
    .into()
}