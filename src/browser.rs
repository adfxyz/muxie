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
        command.args(&self.args);
        command.arg(url);
        command.spawn()?;
        Ok(())
    }
}
