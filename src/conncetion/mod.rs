use std::io::Error;

use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::TcpStream,
};
use tokio_postgres::types::private::BytesMut;

use crate::{cmd::Cmd, frame::Frame};

pub struct Conncetion {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    frame: Frame,
}

impl Conncetion {
    pub fn new(stream: TcpStream, cmd: &mut Cmd) -> Conncetion {
        Conncetion {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
            frame: cmd.into_frame(),
        }
    }

    pub async fn write_all(&mut self) -> Result<(), Error> {
        //TODO: turn frame in to &[u8] and write to socket
        if let Frame::Array(frames) = &self.frame {
            for frame in frames {
                match frame {
                    Frame::Sign(sign) => {
                        self.stream.write_u8(*sign).await?;
                    }
                    Frame::Bulk(bytes) => {
                        self.stream.write_all(&bytes).await?;
                    }
                    Frame::Interger(n) => {
                        self.stream.write_u8(*n).await?;
                    }
                    Frame::Array(_) => {
                        unreachable!()
                    }
                }
            }

            self.stream.flush().await?;
        }

        Ok(())
    }
}
