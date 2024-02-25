use crate::data::*;

lazy_static! {
    pub(crate) static ref COMMAND_PREFIX: &'static str = "!";
    pub(crate) static ref COMMAND_LIST: Vec<String> = vec![
        format!("{}help", *COMMAND_PREFIX),
        format!("{}ping", *COMMAND_PREFIX),
    ];
}
