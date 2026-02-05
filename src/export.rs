use crate::db::Prompt;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn export_to_txt(prompts: &[Prompt], path: &Path) -> io::Result<usize> {
    let mut file = File::create(path)?;
    let count = prompts.len();

    for prompt in prompts {
        writeln!(file, "{}", prompt.text)?;
    }

    Ok(count)
}
