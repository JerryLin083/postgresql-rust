use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use tokio::{io::AsyncReadExt, net::TcpListener};
use tokio_postgres::{Config, config};

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

    //here use self-verify cer
    builder.set_verify(SslVerifyMode::NONE);

    let connector = MakeTlsConnector::new(builder.build());

    let (client, connection) = config.connect(connector).await.map_err(|err| {
        eprintln!("Error: {}", err);
    })?;

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
            client
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
                        //TODO: hadnle client operation
                        println!("Message from client: {:?}", &buf[..n]);
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
