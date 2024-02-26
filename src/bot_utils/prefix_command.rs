use crate::data::commands::COMMAND_PREFIX;
pub fn prefix_command(command: &str) -> String {
    COMMAND_PREFIX.to_string().to_owned() + command
}
