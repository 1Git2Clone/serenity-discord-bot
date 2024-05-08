#[derive(Debug, Default)]
pub struct CmdPrefixes {
    pub prefixes: Vec<&'static str>,
}
impl CmdPrefixes {
    pub fn set() -> Self {
        Self {
            prefixes: vec!["hu", "ht", "Hu", "hU", "HU", "Ht", "hT", "HT"],
        }
    }
}
fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
#[test]
fn normal_types() {
    _is_normal::<CmdPrefixes>();
}
