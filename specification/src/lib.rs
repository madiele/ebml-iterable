mod test;

///
/// Different data types defined in the EBML specification.
/// 
/// Note: This library made a concious decision to not work with "Date" elements from EBML due to lack of built-in support for dates in Rust. Specification implementations should treat Date elements as Binary so that consumers have the option of parsing the unaltered data using their library of choice, if needed.
/// 

// Possible future feature flag to enable Date functionality by having `chrono` as an optional dependency?
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TagDataType {
    Master,
    UnsignedInt,
    Integer,
    Utf8,
    Binary,
    Float,
}

///
/// This trait should be implemented to define a specification so that EBML can be parsed correctly.  Typically implemented on an Enum of tag variants.
/// 
/// Any specification using EBML can take advantage of this library to parse or write binary data.  As stated in the docs, [TagWriter](https://docs.rs/ebml-iterable/latest/ebml_iterable/struct.TagWriter.html) needs nothing special, but [TagIterator](https://docs.rs/ebml-iterable/latest/ebml_iterable/struct.TagIterator.html) requires a struct implementing this trait.  Custom specification implementations can refer to [webm-iterable](https://crates.io/crates/webm_iterable) as an example.
///

pub trait EbmlSpecification<T: EbmlSpecification<T> + Clone> {
    ///
    /// Pulls the "type" of tag and the tag data type from the spec based on the tag id.
    ///
    /// This function *must* return `None` if the input id is not a `TagSpecificationDataType` in the specification.  Implementors can reference [webm-iterable](https://crates.io/crates/webm_iterable) for an example.
    ///
    fn get_tag_data_type(id: u64) -> Option<TagDataType>;

    
    ///
    /// Gets the id of a specific tag "type".
    ///
    /// Implementors can reference [webm-iterable](https://crates.io/crates/webm_iterable) for an example.
    ///
    fn get_tag_id(item: &T) -> u64 {
        item.get_id()
    }
    
    fn get_unsigned_int_tag(id: u64, data: u64) -> Option<T>;
    
    fn get_signed_int_tag(id: u64, data: i64) -> Option<T>;
    
    fn get_utf8_tag(id: u64, data: String) -> Option<T>;
    
    fn get_binary_tag(id: u64, data: &[u8]) -> Option<T>;
    
    fn get_float_tag(id: u64, data: f64) -> Option<T>;
    
    fn get_master_tag_start(id: u64) -> Option<T>;
    
    fn get_master_tag_end(id: u64) -> Option<T>;
    
    fn get_master_tag_full(id: u64, children: &[T]) -> Option<T>;



    fn get_id(&self) -> u64;

    fn get_unsigned_int_data(&self) -> Option<&u64>;

    fn get_signed_int_data(&self) -> Option<&i64>;

    fn get_utf8_data(&self) -> Option<&str>;

    fn get_binary_data(&self) -> Option<&[u8]>;

    fn get_float_data(&self) -> Option<&f64>;

    fn get_master_data(&self) -> Option<&Master<T>>;


// ///
// /// Gets the type of data that is stored in a specific tag "type".
// ///
// /// The default implementation simply calls [`get_tag_id`] followed by [`get_tag`] and maps the result as the return value.  This function *must* return `None` if the input id is not a `TagSpecificationDataType` in the specification.
// ///
// /// [`get_tag_id`]: #method.get_tag_id
// /// [`get_tag`]: #method.get_tag
//     fn get_tag_data_type(item: &T) -> Option<TagDataType> {
//         <T>::get_tag(<T>::get_tag_id(item)).map(|tag: (T, TagDataType)| tag.1)
//     }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Master<T: EbmlSpecification<T> + Clone> {
    Start,
    End,
    Full(Vec<T>),
}