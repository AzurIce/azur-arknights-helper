
use super::host::Host;

pub mod host_service;
pub mod local_service;

pub trait AdbCommand {
    type Output;

    fn raw_command(&self) -> String;

    fn handle_response(&self, host: &mut Host) -> Result<Self::Output, String>;
}
