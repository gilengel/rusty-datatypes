use crate::structs::DatatypeAttribute;
use crate::structs::DatatypeAttributeType;

pub mod parse;
pub mod structs;
pub mod generate;
pub mod validate;

pub(crate) fn update(attrs: &mut Vec<DatatypeAttribute>) {
    reorder_positions_increasing(attrs)
}

fn reorder_positions_increasing(attrs: &mut Vec<DatatypeAttribute>) {
    let number_of_serialization_members = attrs.iter().fold(0u32, |acc, x| acc + match x.ty {
        
        DatatypeAttributeType::Collection(_, _) => 2,
        _ => 1
    });
    let mut positions: Vec<u32> = (0..number_of_serialization_members).collect();
    attrs
        .iter()
        .filter(|attribute| attribute.position.is_some())
        .for_each(|attribute| positions.retain(|position| { *position != attribute.position.unwrap() as u32 }));


        
    attrs
        .iter_mut()
        .filter(|attribute| attribute.position.is_none())
        .for_each(|attribute| { 
            let lowest_free_position = positions.remove(0);
            attribute.position = Some(lowest_free_position as u8); 
        });

    attrs.sort_by(|a, b| a.position.unwrap().cmp(&b.position.unwrap()));
}