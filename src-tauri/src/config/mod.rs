pub mod verge;
pub mod clash;

pub fn init_config() -> Result<(), String> {
    verge::init_config()?;
    clash::init_config()?;
    Ok(())
}