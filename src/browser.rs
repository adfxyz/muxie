use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Browser {
    pub name: String,
    pub executable: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Browser {
    pub fn open_url(&self, url: &str) -> Result<()> {
        let mut command = std::process::Command::new(&self.executable);
        let mut url_arg_found = false;
        for arg in &self.args {
            match arg.as_str() {
                "%u" | "%U" => {
                    url_arg_found = true;
                    command.arg(url);
                }
                _ => {
                    command.arg(arg);
                }
            }
        }
        if !url_arg_found {
            command.arg(url);
        }
        command.spawn()?;
        Ok(())
    }
}
