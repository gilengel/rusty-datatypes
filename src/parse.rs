use proc_macro_error::abort;
use quote::ToTokens;
use syn::Type;





pub(crate) fn is_collection_type(ty: &Type) -> Option<(bool, Option<u8>)> {
    match ty {
        Type::Path(p) => {
            let segments = &p.path.segments;
            
            if let Some(first) = segments.first() {

                let t = first.ident.to_string();
                let t = t.as_str();

                match t {
                    "Vec" | 
                    "VecDeque" | 
                    "LinkedList" |
                    "HashMap" | 
                    "BTreeMap" | 
                    "HashSet" | 
                    "BTreeSet" |
                    "BinaryHeap" => return Some((true, None)),
                    _ => return None
                }
            }

            Some((true, None))
        },
        Type::Array(array) => {
            let i = array.len.to_token_stream().to_string().parse::<u8>().unwrap();
            
            Some((true, Some(i)))
        }
        _ => None,
    }
}



pub fn get_collection_embedded_type(ty: &Type) -> (String, Option<u8>) {
    match ty {
        Type::Path(p) => {
            let segments = &p.path.segments;
           
            if let Some(first) = segments.first() {
                match &first.arguments {
                    syn::PathArguments::None => todo!(),
                    // <> like Vec, HashMap etc
                    syn::PathArguments::AngleBracketed(x) => {
                        return (x.args.first().unwrap().to_token_stream().to_string(), None);
                    },

                    // () like groups (u32, String, ...)
                    syn::PathArguments::Parenthesized(_) => abort!(ty, "Group datatypes are currently not supported"),
                }
                

            }

           return  ("".to_string(), None);
        },
        Type::Array(array) => { 
            let i = array.len.to_token_stream().to_string().parse::<u8>().unwrap();

            let ty = array.elem.to_token_stream().to_string();
            
            (ty, Some(i))
        },
        _ => todo!(),
    }
}