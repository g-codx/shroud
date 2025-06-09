use std::io::Write;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

pub struct XrayController {
    child: Arc<Mutex<Option<Child>>>,
}

impl XrayController {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self, config: &str) -> Result<(), String> {
        let mut child = Command::new("./xray/xray")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(config.as_bytes())
                .map_err(|e| e.to_string())?;
        }

        *self.child.lock().unwrap() = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            return child.kill().map_err(|e| e.to_string());
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}
