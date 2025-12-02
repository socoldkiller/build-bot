use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::{
    io::{AsyncWriteExt, BufReader, Error, ErrorKind},
    process::Command,
    select,
    sync::mpsc,
};
use uuid::Uuid;

use crate::tty::chunk::{AsyncReadExt2, Chunk};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum TTyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Something went wrong")]
    Other,
}

impl From<TTyError> for std::io::Error {
    fn from(error: TTyError) -> Self {
        match error {
            TTyError::Io(io_error) => io_error,
            TTyError::InvalidInput(msg) => {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, msg)
            }
            TTyError::Other => {
                std::io::Error::new(std::io::ErrorKind::Other, "Something went wrong")
            }
        }
    }
}

pub struct TTy {
    stdout_rx: mpsc::Receiver<String>,
    stderr_rx: mpsc::Receiver<String>,
    stdin: tokio::process::ChildStdin,
    delim: String,
}

async fn read_frames<R>(chunk_block: &mut Chunk<BufReader<R>>) -> String
where
    R: AsyncRead + Unpin,
{
    match chunk_block.next_delim().await {
        Ok(Some(chunk)) => chunk,
        _ => String::from(""),
    }
}

impl TTy {
    pub fn new(cmd: String) -> Result<Self, TTyError> {
        let (stdout_tx, stdout_rx) = mpsc::channel(100);
        let (stderr_tx, stderr_rx) = mpsc::channel(100);

        let mut child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        fn take_child_io<T>(opt: Option<T>, name: &str) -> Result<T, Error> {
            opt.ok_or_else(|| Error::new(ErrorKind::Other, format!("no {}", name)))
        }

        let stdin = take_child_io(child.stdin.take(), "stdin")?;
        let stdout = take_child_io(child.stdout.take(), "stdout")?;
        let stderr = take_child_io(child.stderr.take(), "stderr")?;
        let delim = Uuid::new_v4().to_string();
        let mut delim_clone = delim.clone();
        tokio::spawn(async move {
            let mut stdout_frames = BufReader::new(stdout).chunk(delim_clone.as_mut());
            let mut stderr_frames = BufReader::new(stderr).chunk(delim_clone.as_mut());
            let mut read_loop = async move || -> Result<(), Error> {
                loop {
                    select! {
                        frame = read_frames(&mut stdout_frames) => {
                            stdout_tx.send(frame).await.map_err(|_| Error::new(ErrorKind::Other, "channel closed"))?;
                        }

                        frame = read_frames(&mut stderr_frames) => {
                            stderr_tx.send(frame).await.map_err(|_| Error::new(ErrorKind::Other, "channel closed"))?;
                        }

                        _ = sleep(Duration::from_mins(5)) => {
                            return Err(Error::new(ErrorKind::TimedOut, "read timeout"));
                        }


                    }
                }
            };

            match read_loop().await {
                Ok(_) => println!("done"),
                Err(err) => println!("error: {}", err),
            }
        });

        Ok(Self {
            stdout_rx,
            stderr_rx,
            stdin,
            delim,
        })
    }

    pub async fn write<S: Into<String>>(&mut self, s: S) -> tokio::io::Result<()> {
        let stdin = s.into();
        let output = format!("{}; echo {}; echo {} 1>&2\n", stdin, self.delim, self.delim);
        self.stdin.write_all(output.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<String, Error> {
        let stdout = self
            .stdout_rx
            .recv()
            .await
            .ok_or_else(|| Error::new(ErrorKind::Other, "broken pipe"))?;

        let stderr = self
            .stderr_rx
            .recv()
            .await
            .ok_or_else(|| Error::new(ErrorKind::Other, "broken pipe"))?;

        let output = format!("{}{}", stdout, stderr);
        Ok(output.trim().to_string())
    }
}

#[cfg(test)]
mod tests {}
