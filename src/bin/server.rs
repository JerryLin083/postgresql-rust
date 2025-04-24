use std::sync::Arc;

use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use tokio::sync::Mutex;
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_postgres::{Client, Config, config};

use postgresql_rust::cmd::{self, Cmd, Method};

type Result<T> = std::result::Result<T, ()>;

#[tokio::main]
async fn main() -> Result<()> {
    //TODO: error handle

    //load var from .enc
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

    let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|err| {
        eprintln!("Error: {}", err);
    })?;

    //here use self-signed cer
    builder.set_verify(SslVerifyMode::NONE);

    let connector = MakeTlsConnector::new(builder.build());

    let (client, connection) = config.connect(connector).await.map_err(|err| {
        eprintln!("Error: {}", err);
    })?;

    //TODO: replace stupid single mutex lock by pooled connection
    let arc_client = Arc::new(Mutex::new(client));

    //handle connection
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            eprintln!("Connecttion error: {}", err);
        }
    });

    //listen to socket connection
    let listener = TcpListener::bind("127.0.0.1:8000").await.map_err(|err| {
        eprintln!("Server error: {}", err);
    })?;

    loop {
        if let Ok((mut stream, addr)) = listener.accept().await {
            println!("socket connected");

            let client = arc_client.clone();

            client
                .try_lock()
                .unwrap()
                .execute(
                    "insert into connection(addr) values($1)",
                    &[&addr.to_string()],
                )
                .await
                .map_err(|err| {
                    eprintln!("Insert error: {}", err);
                })?;

            tokio::spawn(async move {
                let mut buf = vec![0; 4 * 1024];
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        println!("disconnection from {:?}", addr)
                    }
                    Ok(n) => {
                        let mut cmd = cmd::Cmd::from_vec(&buf[0..n]);
                        let _ = handle_cmd(&mut cmd, client.clone()).await;
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

async fn handle_cmd(cmd: &mut Cmd, client: Arc<Mutex<Client>>) -> Result<()> {
    match cmd.method {
        Method::Query => {
            let rows = client
                .try_lock()
                .unwrap()
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
                .try_lock()
                .unwrap()
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
                .try_lock()
                .unwrap()
                .execute(&cmd.update_execution(), &[])
                .await
                .map_err(|err| eprintln!("Error: {}", err))?;

            println!("{} row(s) was updated", rows);
        }
        Method::Delete => {
            let rows = client
                .try_lock()
                .unwrap()
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
