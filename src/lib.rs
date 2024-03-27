extern crate proc_macro;

#[macro_use]
extern crate quote;
extern crate syn;

mod parse;
mod structs;
mod data_enum;
mod data_struct;
mod types;

use data_enum::structs::DataEnumArgs;
use data_struct::{parse::ItemStruct, validate::validate, update};
use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use structs::DatatypeAttribute;
use syn::{DeriveInput, parse_macro_input};

/// Convenience macro that is capable of generating (de-)serialize functions
/// to binary and vice versa. It was designed with sdk datatypes in mind but can also
/// be used for any other datatypes.
///
/// # Usage
/// 
/// Simple struct where all attributes shall be serialized and deserialized:
/// Per default all member within a struct are then (de-)serialized.
/// 
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     message: String
/// }
/// ```
/// 
///
/// ## Skipping (De-)Serialization
/// You can skip the generation of (de-)serialize function by omitting the corresponding keyword.
/// This example skips deserialization
///
/// ```rust
/// #[datatype(serialize)]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     message: String
/// }
/// ```
///
/// ## Endianness
/// Usually all attributes are (de-)serialized using big endianess but you can change this
/// per attribute (only for numbers and datatypes that are not u8 and i8):
///
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     #[field(endiness=LittleEndian)]
///     seconds: u16,
///     minutes: u16,
///     hours:   u16,
///     message: String
/// }
/// ```
///
/// It is also possible to change the endianess for all fields like this:
///
/// ```rust
/// #[datatype(endianness=LittleEndian)]
/// pub struct TimedMessage
/// {
///     seconds: u16,
///     minutes: u16,
///     hours:   u16,
///     message: String
/// }
/// ```
/// ## Collections
/// Collections are usually (de-)serialized by an additional field of type u16 for the
/// collection length. This can be adapted per attribute:
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     #[field(endiness=LittleEndian)]
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     
///     #[field(length_type=u8)]
///     messages: Vec<String>
/// }
/// ```
/// ## Reserved Fields
/// Sometimes your datatype might need reserved fields where the actual content does not matter
/// but is important for (de-)serialization in order to fulfill already existing interface
/// specifications. In this case simply use the reserved attribute. 
/// 
/// Keep in mind that reserved datafields are omitted in the generated datatype. Therefore you
/// cannot access the _reserved0 and _reserved1 fields on the struct as they do not exist for your
/// code.
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     
///     #[field(reserved)]
///     _reserved0: u8,
/// 
///     #[field(reserved)]
///     _reserved1: u8,
/// }
/// ```
///
/// You can also group multiple reserved bytes into one attribute to make your code easier to read.
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     
///     #[field(reserved)]
///     _reserved: u16,
///
///     messages: Vec<String>
/// }
/// ```
/// ⚠️ You can use the collection datatype Vec to group multiple reserved bytes but you to need to use ```reserved```
/// keyword - otherwise an extra field for the length is generated making your (de-)serialize functions most likely 
/// not compatible as expected.
/// 
/// CURRENTLY NOT IMPLEMENTED
/// 
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///     
///     #[field(reserved)]
///     _reserved: Vec<u8>
///
///     messages: Vec<String>
/// }
/// ```
/// ## Aggregated datatypes
/// It is possible and encouraged to use reuse datatypes in another ones.
/// The only thing you need to ensure, that each struct uses the ```Datatype``` macro:
/// ```rust
/// #[datatype]
/// pub struct Time
/// {
///     #[field(endiness=LittleEndian)]
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
/// }
///
/// #[datatype]
/// pub struct TimedMessage
/// {
///     time: Time,
///     messages: Vec<String>
/// }
/// ```
/// ## Alternativ positioning
/// Attributes are (de-)serialized per default in the order how they are defined in a struct with the
/// topmost being the first and the bottommost the last one that are (de-)serialized. You can
/// specify an alternate position to comply with legacy definitions by using the ```position``` tag:
/// ```rust
/// #[datatype]
/// pub struct Time
/// {
///     #[field(position=1)]
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
/// }
/// ```
/// In the above example the order of the attributes is;
/// * minutes
/// * seconds
/// * hours
///
/// If you need (de-)serialize the length of a container at a different position you can use the ```length_position``` tag:
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///
///     #[field(length_position=1)]
///     messages: Vec<String>
/// }
/// ```
/// In the above example the order is:
/// * seconds
/// * length of messages
/// * minutes
/// * hours
///
/// ## Custom (de-)serialize function
/// 
/// CURRENTLY NOT IMPLEMENTED
/// 
/// ```rust
/// #[datatype]
/// pub struct TimedMessage
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///
///     #[field(serialize=serialize_func, deserialize=deserialize_func)]
///     messages: Vec<String>
/// }
/// ```
/// 
/// ## Ignoring attributes
/// Per default all attributes are used for the (de-)serialization. You can disable the generation for
/// single attributes completly (like if you need an extra attribute holding a converted datatype for example)
/// by using the ```ignore``` keyword
/// 
/// CURRENTLY NOT IMPLEMENTED
/// 
/// ```rust
/// pub struct Time
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
///
///     [#field(ignore)]
///     iso_string: String
/// }
/// ```
/// 
/// ## Conditional (de-)serializing
/// It is possible to avoid (de-)serializing based on a condition by using the ```conditional``` keyword. In the
/// following example the attribute iso_string is only used for (de-)serialization if the ```format``` has the value of ```TimeFormat::MMSS```.
/// You can use every boolean expression as a conditional as long as it is valid rust code and only contains attributes that are (de-)serialized
/// at the point of checking the conditional.
/// 
/// CURRENTLY NOT IMPLEMENTED
/// 
/// ```rust
/// pub enum TimeFormat {
///     HHMMSS,
///     MMSS,
///     HHMM
/// }
/// pub struct Time
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
/// 
///     format:  TimeFormat,
///
///     [#field(conditional(format=TimeFormat::MMSS))]
///     iso_string: String
/// }
/// ```
/// 
/// ## Versioning
/// Versioning is completly optional and done on datatype level meaning if enabled, for each datatype that is serialized 
/// a corresponding version is saved. By using the ```version``` attribute you then specify for single members for 
/// which version they are available:
/// 
/// CURRENTLY NOT IMPLEMENTED
/// 
/// ```rust
/// pub struct Time
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
/// 
///     format:  TimeFormat,
///
///     [#field(version(min=0, max=3))]
///     iso_string: String,
/// 
///     [#field(version(min=3))
///     iso: String 
/// }
/// ```
///
/// ## Fixed Size
/// If your datatype always needs to have a fixed size you can enforce it by using the ```size``` attribute on the struct and one ```variable``` attribute
/// on a data field. The ```size``` attribute specifies the size of the datatype in bits. Fields with the ```variable``` must be of type ```Vec<char>```.
/// 
/// ```rust
/// #[datatype(fixed_size=128)]
/// pub struct VariableTime
/// {
///     seconds: u8,
///     minutes: u8,
///     hours:   u8,
/// 
///     format:  TimeFormat,
///
///     [#field(variable))]
///     padding: Vec<char>,
/// 
///     iso_string: String,
/// }
/// ```

#[proc_macro_error]
#[proc_macro_attribute]
pub fn datatype(_: TokenStream, input: TokenStream) -> TokenStream {
    

    let datatype_struct = parse_macro_input!(input as ItemStruct);
    let mut attrs: Vec<DatatypeAttribute> = datatype_struct.fields
                                            .iter()
                                            .map(|x| Into::<Vec<DatatypeAttribute>>::into(x))
                                            .flatten()
                                            .collect();

    // checks that position attributes are valid (within range, no duplicates)
    validate(&attrs);

    // reorders the attributes in increasing order
    update(&mut attrs); 

    // Build the impl
    data_struct::generate::produce(&datatype_struct, &attrs).into()
}

/// Convenience macro that is capable of generating (de-)serialize functions
/// to binary and vice versa for enums. It is designed to work alongside ```Datatype```.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn data_enum(args: TokenStream, input: TokenStream) -> TokenStream {

    let ast: DeriveInput = syn::parse(input.clone()).expect("Couldn't parse for datatype");
    let args = parse_macro_input!(args as DataEnumArgs);
   
    data_enum::generate::produce(&ast, &args).into()
}




