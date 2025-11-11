use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Request = 0,
    Response = 1,
    Event = 2,
}

impl From<Kind> for u8 {
    fn from(k: Kind) -> Self {
        k as u8
    }
}

pub struct FrameHeader {
    pub api_major: u8,
    pub kind: Kind,
    pub schema_id: u32,
    pub len: u32,
}

impl FrameHeader {
    pub const SIZE: usize = 1 + 1 + 4 + 4;
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut b = [0u8; Self::SIZE];
        b[0] = self.api_major;
        b[1] = self.kind.into();
        b[2..6].copy_from_slice(&self.schema_id.to_be_bytes());
        b[6..10].copy_from_slice(&self.len.to_be_bytes());
        b
    }
    pub fn from_bytes(b: [u8; 10]) -> Self {
        Self {
            api_major: b[0],
            kind: match b[1] {
                0 => Kind::Request,
                1 => Kind::Response,
                _ => Kind::Event,
            },
            schema_id: u32::from_be_bytes(b[2..6].try_into().unwrap()),
            len: u32::from_be_bytes(b[6..10].try_into().unwrap()),
        }
    }
}

pub async fn write_payload<W: AsyncWrite + Unpin>(mut w: W, hdr: FrameHeader, buf: &[u8]) -> std::io::Result<()> {
    debug_assert_eq!(hdr.len as usize, buf.len());
    w.write_all(&hdr.to_bytes()).await?;
    w.write_all(buf).await?;
    w.flush().await
}

pub async fn read_payload<R: AsyncRead + Unpin>(mut r: R) -> std::io::Result<(FrameHeader, Vec<u8>)> {
    let mut hdr = [0u8; FrameHeader::SIZE];
    r.read_exact(&mut hdr).await?;
    let hdr = FrameHeader::from_bytes(hdr);
    let mut buf = vec![0u8; hdr.len as usize];
    r.read_exact(&mut buf).await?;
    Ok((hdr, buf))
}
