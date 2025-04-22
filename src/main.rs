mod cmd;
mod conncetion;
mod frame;

use std::io::Error;

use cmd::Cmd;
use conncetion::Conncetion;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;

    //TODO: use cmd to generate syntax
    let mut cmd = Cmd::build(cmd::Method::Query);
    cmd.set_table("test")
        .set_column(vec!["first_name", "last_name"])
        .set_condition("order by first name limit 10")
        .into_frame();

    let mut conncetion = Conncetion::new(stream, &mut cmd);

    conncetion.write_all().await;

    //TODO: try parse from frame

    Ok(())
}
