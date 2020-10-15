use std::error::Error;
use talys_empire::Core;

fn main() -> Result<(), Box<dyn Error>> {
    let core = Core::new("dat");
    core.convert()?;
    Ok(())
}
