use postgresql_rust::{
    cmd::{self, Cmd},
    conncetion::Conncetion,
};

use std::io::Error;

use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let stream = TcpStream::connect("127.0.0.1:8000").await?;

    let mut cmd = Cmd::build(cmd::Method::Update);
    cmd.set_table("customer")
        .set_columns(vec!["first_name", "last_name"])
        .set_values(vec!["berry", "lin"])
        .set_condition("where first_name = 'Berry'")
        .into_frame();

    let mut conncetion = Conncetion::new(stream, &mut cmd);

    conncetion.write_all().await?;

    //TODO: try parse from frame

    Ok(())
}
