#[cfg(test)]
fn sized_send_sunc_unpin<T: Sized + Send + Sync + Unpin>() {}

#[test]
fn normal_types() {
    use crate::data::command_data::Data;
    use crate::enums::command_enums::EmbedType;
    use crate::enums::schemas::DatabaseSchema;

    sized_send_sunc_unpin::<Data>();
    sized_send_sunc_unpin::<EmbedType>();
    sized_send_sunc_unpin::<DatabaseSchema>();
}
