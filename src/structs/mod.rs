#[derive(Debug, Default)]
pub struct CmdPrefixes {
    pub prefixes: Vec<&'static str>,
}
impl CmdPrefixes {
    pub fn set() -> Self {
        Self {
            prefixes: vec!["hu", "Hu", "hU", "HU", "ht", "Ht", "hT", "HT"],
        }
    }
}
