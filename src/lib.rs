use std::error::Error;
use std::{fs, io};

const TRANSITIONS: &str = "114Cd_cupture_gamma_spectra.dat";

pub struct Core {
    directory: String,
}

impl Core {
    pub fn new(dir: &str) -> Self {
        Self {
            directory: String::from(dir),
        }
    }

    fn read_transitions(&self) -> io::Result<String> {
        let data = fs::read_to_string(format!("{}/{}", self.directory, TRANSITIONS))?;
        Ok(data)
    }

    pub fn convert(&self) -> Result<(), Box<dyn Error>> {
        let transitions = self.read_transitions()?;

        Ok(())
    }
}
