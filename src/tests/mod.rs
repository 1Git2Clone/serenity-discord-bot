#[cfg(test)]

/// This is a useful way to test if your structs can be syncronized.
/// Take for example using Rc<T> instead of Arc<T>
/// It'll give an error on compile time telling you that you can't synchronize the data safely.
///
/// NOTE - The Arc<T> vs Rc<T> example won't work with this exact code sample because both of
/// them can't be serialized. (serde::Serialize)
/// In order to use data that can't be serialized or deserlialized you need to do the following:
///
/// ```rust
/// pub struct Data {
///     // Existing data...
///     #[cfg_attr(feature = "serde", serde(skip))]
///     pub some_unserializable_data: std::sync::Arc<i32>,
/// }
/// ```
///
/// Tutorial vid for the topic:
/// https://www.youtube.com/watch?v=Nzclc6MswaI
fn _is_normal<T: Sized + Send + Sync + Unpin>() {}

#[test]
fn normal_types() {
    use crate::data::command_data::Data;
    use crate::enums::command_enums::EmbedType;
    use crate::enums::schemas::DatabaseSchema;
    use crate::structs::CmdPrefixes;

    _is_normal::<Data>();
    _is_normal::<EmbedType>();
    _is_normal::<DatabaseSchema>();
    _is_normal::<CmdPrefixes>();
}
