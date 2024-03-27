use std::collections::HashMap;

use proc_macro_error::emit_error;

use crate::structs::DatatypeAttribute;

pub(crate) fn validate(attrs:& Vec<DatatypeAttribute>)
{

    check_for_invalide_indices(&attrs);
    check_for_double_indices(&attrs);
}


fn check_for_invalide_indices(attrs: &Vec<DatatypeAttribute>) {
    let number_of_attributes = attrs.len();
    attrs.iter().filter(|x| x.position.is_some()).for_each(|x| {
        let position = x.position.unwrap();
        
        if position > number_of_attributes as u8 {
            emit_error!(
                x.position_span.unwrap(),
                "Invalid position: The datatype has only {} attributes making {} the max allowed position. You tried to use {} for the attribute.",
                number_of_attributes + 1, number_of_attributes, position
            );
        }
    });    
}

fn check_for_double_indices(attrs: &Vec<DatatypeAttribute>) {
    let mut m: HashMap<u8, Vec<&DatatypeAttribute>> = HashMap::default();
    
    
    for a in attrs.iter().filter(|x| x.position.is_some() ) {
        let position = a.position.unwrap();
        match m.contains_key(&position) {
            true => {
                m.get_mut(&position).unwrap().push(&a);
            }
            false => {
                m.insert(position, vec![&a]);
            }
        }
    }

    if let Some((position, matches)) = m.iter().find(|(_, attributes)| attributes.len() == 2) {
        for m in matches {
            emit_error!(
                m.position_span.unwrap(),
                "Duplicate position: The position {} is used for another attribute",
                position
            );
        }
    }
}