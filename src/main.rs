use anyhow::Result;
use fswhy::App;
use std::env;
use std::path::PathBuf;
fn main() -> Result<()> {
    let root_path: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let app = App::new(root_path)?;
    app.run()?;
    Ok(())
}
