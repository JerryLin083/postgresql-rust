use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres::Config;
use postgres_openssl::MakeTlsConnector;

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    //load dotenv
    dotenv().map_err(|err| eprintln!("ERROR: {}", err))?;

    //TODO: use tls connection
    let mut config = Config::new();
    config
        .host(&dotenv::var("HOST").unwrap_or(String::new()))
        .user(&dotenv::var("USER").unwrap_or(String::new()))
        .password(&dotenv::var("PASSWORD").unwrap_or(String::new()))
        .dbname(&dotenv::var("DBNAME").unwrap_or(String::new()))
        .ssl_mode(postgres::config::SslMode::Require);

    let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|err| {
        eprintln!("Error: {}", err);
    })?;
    builder.set_verify(SslVerifyMode::NONE);

    let connector = MakeTlsConnector::new(builder.build());

    let mut client = config.connect(connector).map_err(|err| {
        eprintln!("Error: {}", err);
    })?;

    let rows = client
        .query(
            "select first_name, last_name from customer order by first_name limit 10",
            &[],
        )
        .map_err(|err| {
            eprintln!("Error: {}", err);
        })?;

    for row in rows {
        let first_name: &str = row.get(0);
        let last_name: &str = row.get(1);

        println!("Customer name: {} {}", first_name, last_name);
    }

    Ok(())
}
