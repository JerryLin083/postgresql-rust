use postgresql_rust::{
    Result,
    cmd::{self, Cmd, Method},
    pg_pool::{PgConnection, PgPool},
};
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_postgres::{Config, config};

#[tokio::main]
async fn main() -> Result<()> {
    //connection config
    let mut config = Config::new();
    config
        .host(&dotenv::var("HOST").unwrap())
        .user(&dotenv::var("USER").unwrap())
        .password(&dotenv::var("PASSWORD").unwrap())
        .dbname(&dotenv::var("DBNAME").unwrap())
        .ssl_mode(config::SslMode::Require);

    let pg_pool = PgPool::build(config, 2).await;

    //listen to socket connection
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("socket connected");

        let pg_connection = pg_pool.get_connection().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 4 * 1024];
            match stream.read(&mut buf).await {
                Ok(0) => {
                    println!("disconnection from {:?}", addr)
                }
                Ok(n) => {
                    pg_connection
                        .get_client()
                        .execute(
                            "insert into connection(addr) values($1)",
                            &[&addr.to_string()],
                        )
                        .await
                        .expect("Failed to insert connection info");

                    let mut cmd = cmd::Cmd::from_vec(&buf[0..n]);

                    if let Err(err) = handle_cmd(&mut cmd, pg_connection).await {
                        eprintln!("Fail to execute command error: {}", err);
                    };
                }
                Err(err) => {
                    eprintln!("Socket connection error: {}", err);
                }
            };
        });
    }
}

async fn handle_cmd(cmd: &mut Cmd, pg_connection: PgConnection) -> Result<()> {
    let client = pg_connection.get_client();

    match cmd.method {
        Method::Query => {
            let rows = client.query(&cmd.query_execution(), &[]).await?;

            for row in rows {
                let first_name: &str = row.get(0);
                let last_name: &str = row.get(1);

                println!("name: {} {}", first_name, last_name);
            }
        }
        Method::Insert => {
            let result = client.execute(&cmd.insert_execution(), &[]).await?;

            if result == 1 {
                println!("insert successfully")
            } else {
                println!("insert failed")
            }
        }
        Method::Update => {
            let rows = client.execute(&cmd.update_execution(), &[]).await?;

            println!("{} row(s) was updated", rows);
        }
        Method::Delete => {
            let rows = client.execute(&cmd.delete_execution(), &[]).await?;

            println!("{} row(s) was deleted", rows);
        }
        Method::Null => {}
    }

    Ok(())
}
