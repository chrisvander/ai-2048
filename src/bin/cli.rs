use ai_2048;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    ai_2048::tui::start()?;
    Ok(())
}
