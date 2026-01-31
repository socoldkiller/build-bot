use futures_util::TryFutureExt;
use std::ops::{Deref, DerefMut};
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

use crate::tty::chunk::{AsyncReadChunk, Chunk};
use thiserror::Error;
use tokio::process::Child;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum TTyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Something went wrong")]
    Other,

    #[error("read timeout")]
    Timeout,

    #[error("Closed")]
    Closed,
}

struct BotChild {
    inner: Child,
    tx: Option<Sender<String>>,
}

impl Deref for BotChild {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for BotChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<Child> for BotChild {
    fn from(inner: Child) -> Self {
        Self { inner, tx: None }
    }
}

impl Drop for BotChild {
    fn drop(&mut self) {
        match self.tx.take() {
            Some(tx) => {
                let msg = format!("pid {} closed", self.inner.id().unwrap_or(0));
                match tx.send(msg) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error sending shutdown message: {}", e);
                    }
                }
            }
            None => panic!("sender dropped while receiver dropped"),
        }
    }
}

impl BotChild {
    fn set_tx(&mut self, tx: Sender<String>) {
        self.tx = Some(tx);
    }
}

impl From<TTyError> for Error {
    fn from(error: TTyError) -> Self {
        match error {
            TTyError::Io(io_error) => io_error,
            TTyError::InvalidInput(msg) => Error::new(ErrorKind::InvalidInput, msg),
            TTyError::Other => Error::new(ErrorKind::Other, "Something went wrong"),
            TTyError::Timeout => Error::new(ErrorKind::TimedOut, "read timeout"),
            TTyError::Closed => Error::new(ErrorKind::InvalidInput, "closed"),
        }
    }
}

pub struct BotTTy {
    stdout_rx: mpsc::Receiver<String>,
    stderr_rx: mpsc::Receiver<String>,
    stdin: tokio::process::ChildStdin,
    delim: String,
}

async fn read_frames<R>(chunk_block: &mut Chunk<BufReader<R>>) -> Result<Option<String>, Error>
where
    R: AsyncRead + Unpin,
{
    match chunk_block.next_delim().await {
        Ok(Some(chunk)) => Ok(Some(chunk.trim().to_owned())),
        Ok(None) => Ok(None), // EOF
        Err(e) => Err(e),
    }
}

impl BotTTy {
    pub fn new<C: Into<String>>(cmd: C) -> Result<Self, TTyError> {
        let (stdout_tx, stdout_rx) = mpsc::channel(100);
        let (stderr_tx, stderr_rx) = mpsc::channel(100);

        let (close_tx, mut close_rx) = oneshot::channel();

        let mut child: BotChild = Command::new(cmd.into())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .into();
        child.set_tx(close_tx);

        fn take_child_io<T>(opt: Option<T>, name: &str) -> Result<T, Error> {
            opt.ok_or_else(|| Error::new(ErrorKind::Other, format!("no {}", name)))
        }
        let stdin = take_child_io(child.stdin.take(), "stdin")?;
        let stdout = take_child_io(child.stdout.take(), "stdout")?;
        let stderr = take_child_io(child.stderr.take(), "stderr")?;

        tokio::spawn(async move {
            child.wait().await.unwrap();
        });

        let delim = Uuid::new_v4().to_string();
        let d = delim.to_owned();

        tokio::spawn(async move {
            let mut stdout_frames = BufReader::new(stdout).chunk(delim.as_ref());
            let mut stderr_frames = BufReader::new(stderr).chunk(delim.as_ref());
            let mut read_loop = async || -> Result<(), TTyError> {
                loop {
                    select! {
                        biased;

                        _ = &mut close_rx => {
                            return Err(TTyError::Closed);
                        }

                        frame = read_frames(&mut stdout_frames) => {
                            match frame? {
                                Some(frame_str) => {
                                    stdout_tx.send(frame_str).await
                                        .map_err(|_| Error::new(ErrorKind::Other, "channel closed"))?;
                                }
                                None => {
                                   return Err(TTyError::Other);
                                }
                            }
                        }

                        frame = read_frames(&mut stderr_frames) => {
                            match frame? {
                                Some(frame_str) => {
                                    stderr_tx.send(frame_str).await
                                        .map_err(|_| Error::new(ErrorKind::Other, "channel closed"))?;
                                }
                                None => {
                                   return Err(TTyError::Other);
                                }
                            }
                        }

                        _ = sleep(Duration::from_mins(5)) => {
                            return Err(TTyError::Timeout);
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
            delim: d,
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
            .ok_or_else(|| Error::new(ErrorKind::BrokenPipe, "broken pipe"))?;

        let stderr = self
            .stderr_rx
            .recv()
            .await
            .ok_or_else(|| Error::new(ErrorKind::BrokenPipe, "broken pipe"))?;

        let output = format!("{}{}", stdout, stderr);
        Ok(output.to_owned())
    }
}

#[cfg(test)]
mod tests {}
