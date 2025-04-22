use postgresql_rust::{
    cmd::{self, Cmd},
    conncetion::Conncetion,
};

use std::io::Error;

use tokio::{io::AsyncWriteExt, net::TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;

    //TODO: use cmd to generate syntax
    let mut cmd = Cmd::build(cmd::Method::Insert);
    cmd.set_table("test")
        .set_column(vec!["first_name", "last_name"])
        .set_values(vec!["jerry", "lin"])
        .set_condition("order by first_name limit 10")
        .into_frame();

    let mut conncetion = Conncetion::new(stream, &mut cmd);

    conncetion.write_all().await?;

    //TODO: try parse from frame

    Ok(())
}
