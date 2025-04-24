use std::sync::Arc;

use dotenv::dotenv;
use postgresql_rust::pg_pool::{PgConnection, PgPool};
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_postgres::{Config, config};

use postgresql_rust::cmd::{self, Cmd, Method};

type Result<T> = std::result::Result<T, ()>;

#[tokio::main]
async fn main() -> Result<()> {
    //TODO: error handle

    //load var from .env
    dotenv().map_err(|err| {
        eprintln!("Error: {}", err);
    })?;

    //connection config
    let mut config = Config::new();

    config
        .host(&dotenv::var("HOST").unwrap_or(String::new()))
        .user(&dotenv::var("USER").unwrap_or(String::new()))
        .password(&dotenv::var("PASSWORD").unwrap_or(String::new()))
        .dbname(&dotenv::var("DBNAME").unwrap_or(String::new()))
        .ssl_mode(config::SslMode::Require);

    let pg_pool = Arc::new(PgPool::build(config, 2).await);

    //listen to socket connection
    let listener = TcpListener::bind("127.0.0.1:8000").await.map_err(|err| {
        eprintln!("Server error: {}", err);
    })?;

    loop {
        if let Ok((mut stream, addr)) = listener.accept().await {
            println!("socket connected");

            let pg_connection = pg_pool
                .get_connection()
                .await
                .expect("Pool connetion error");

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
                            .map_err(|err| {
                                eprintln!("Insert error: {}", err);
                            })
                            .unwrap();

                        let mut cmd = cmd::Cmd::from_vec(&buf[0..n]);
                        let _ = handle_cmd(&mut cmd, pg_connection).await;
                    }
                    Err(err) => {
                        eprintln!("Socket connection error: {}", err);
                    }
                };
            })
        } else {
            continue;
        };
    }
}

async fn handle_cmd(cmd: &mut Cmd, pg_connection: PgConnection) -> Result<()> {
    let client = pg_connection.get_client();

    match cmd.method {
        Method::Query => {
            let rows = client
                .query(&cmd.query_execution(), &[])
                .await
                .map_err(|err| eprintln!("Query Error: {}", err))?;

            for row in rows {
                let first_name: &str = row.get(0);
                let last_name: &str = row.get(1);

                println!("name: {} {}", first_name, last_name);
            }
        }
        Method::Insert => {
            let result = client
                .execute(&cmd.insert_execution(), &[])
                .await
                .map_err(|err| eprintln!("Error: {}", err))?;

            if result == 1 {
                println!("insert successfully")
            } else {
                println!("insert failed")
            }
        }
        Method::Update => {
            let rows = client
                .execute(&cmd.update_execution(), &[])
                .await
                .map_err(|err| eprintln!("Error: {}", err))?;

            println!("{} row(s) was updated", rows);
        }
        Method::Delete => {
            let rows = client
                .execute(&cmd.delete_execution(), &[])
                .await
                .map_err(|err| {
                    eprintln!("Error: {}", err);
                })?;

            println!("{} row(s) was deleted", rows);
        }
        Method::Null => {}
    }

    Ok(())
}
