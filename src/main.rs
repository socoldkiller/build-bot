
mod tty;

use tokio::io::{AsyncBufReadExt, BufReader, Error};
use crate::tty::tty::TTy;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut tty = TTy::new("zsh".into())?;

    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = stdin_lines.next_line().await? {
        tty.write(line).await?;
        let output = tty.read().await?;
        println!("{}", output);
    }
    Ok(())
}
