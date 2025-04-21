use postgres::{Client, NoTls};

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    let mut client = Client::connect(
        "host=localhost user=postgres password='j2955687zz2' dbname=dvdrental",
        NoTls,
    )
    .map_err(|err| {
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
