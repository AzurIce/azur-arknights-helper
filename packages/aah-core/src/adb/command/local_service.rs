use super::AdbCommand;

#[cfg(test)]
mod test {
    use crate::adb::host;

    use super::ScreenCap;

    #[test]
    fn test_screencap() {
        let mut host = host::connect_default().unwrap();
        let res = host
            .execute_local_command("127.0.0.1:16384".to_string(), ScreenCap)
            .unwrap();
        println!("{}", res.len())
    }
}

/// shell:command
/// 
/// command is something like "cmd arg1 arg2 ..."
pub struct ShellCommand {
    command: String
}

impl ShellCommand {
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl AdbCommand for ShellCommand {
    type Output = ();

    fn raw_command(&self) -> String {
        format!("shell:{}", self.command)
    }

    fn handle_response(&self, host: &mut crate::adb::host::Host) -> Result<Self::Output, String> {
        host.check_response_status()?;
        Ok(())
    }
}

/// shell:screencap -p
pub struct ScreenCap;

impl AdbCommand for ScreenCap {
    type Output = Vec<u8>;
    fn raw_command(&self) -> String {
        "shell:screencap -p".to_string()
    }
    fn handle_response(&self, host: &mut crate::adb::host::Host) -> Result<Self::Output, String> {
        host.check_response_status()?;
        host.read_to_end()
    }
}
