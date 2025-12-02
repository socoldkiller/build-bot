use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub struct Chunk<R> {
    reader: BufReader<R>,
    delim: Vec<u8>,
    buf: Vec<u8>,
    base: u64,
    pow_base: u64,
    delim_hash: u64,
}

impl<R: AsyncRead + Unpin> Chunk<R> {
    pub fn new(reader: R, delim: &str) -> Self {
        let base: u64 = 257;
        let delim_bytes = delim.as_bytes().to_vec();

        let mut pow_base = 1u64;
        for _ in 1..delim_bytes.len() {
            pow_base = pow_base.wrapping_mul(base);
        }

        let mut h = 0u64;
        for &b in &delim_bytes {
            h = h.wrapping_mul(base).wrapping_add(b as u64);
        }

        Self {
            reader: BufReader::new(reader),
            delim: delim_bytes,
            buf: Vec::new(),
            base,
            pow_base,
            delim_hash: h,
        }
    }

    pub async fn next_delim(&mut self) -> io::Result<Option<String>> {
        let delim_len = self.delim.len();
        if delim_len == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "empty delimiter"));
        }

        let mut window: Vec<u8> = Vec::with_capacity(delim_len);
        let mut window_hash: u64 = 0;

        loop {
            let mut byte = [0u8; 1];
            let n = self.reader.read(&mut byte).await?;
            if n == 0 {
                return if self.buf.is_empty() {
                    Ok(None)
                } else {
                    let frame = std::mem::take(&mut self.buf);
                    Ok(Some(String::from_utf8_lossy(&frame).to_string()))
                }
            }

            self.buf.push(byte[0]);


            if window.len() < delim_len {
                window.push(byte[0]);
                window_hash = window_hash
                    .wrapping_mul(self.base)
                    .wrapping_add(byte[0] as u64);
            } else {
                let out = window.remove(0);
                window.push(byte[0]);
                window_hash = window_hash
                    .wrapping_sub((out as u64).wrapping_mul(self.pow_base))
                    .wrapping_mul(self.base)
                    .wrapping_add(byte[0] as u64);
            }


            if window.len() == delim_len && window_hash == self.delim_hash {
                if &self.buf[self.buf.len() - delim_len..] == self.delim.as_slice() {
                    let frame_len = self.buf.len() - delim_len;
                    let frame = self.buf[..frame_len].to_vec();
                    self.buf.drain(..frame_len + delim_len);
                    return Ok(Some(String::from_utf8_lossy(&frame).to_string()));
                }
            }
        }
    }
}

pub trait AsyncReadChunk: AsyncRead {
    fn chunk(self, delim: &str) -> Chunk<Self>
    where
        Self: Sized,
        Self: Unpin,
    {
        Chunk::new(self, delim)
    }
}

impl<T: AsyncRead + ?Sized> AsyncReadChunk for T {}
