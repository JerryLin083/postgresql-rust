use std::{collections::VecDeque, sync::Arc};

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use tokio::sync::{AcquireError, Mutex, OwnedSemaphorePermit, Semaphore};
use tokio_postgres::{Client, Config};

#[derive(Debug)]
pub struct PgPool {
    clients: Arc<Mutex<VecDeque<Client>>>,
    semaphore: Arc<Semaphore>,
}

impl PgPool {
    pub async fn build(config: Config, size: usize) -> Self {
        let mut clients = VecDeque::with_capacity(size);

        for _ in 0..size {
            let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
            builder.set_verify(SslVerifyMode::NONE); //here use self-signed cert

            let connector = MakeTlsConnector::new(builder.build());

            match config.connect(connector).await {
                Ok((client, connection)) => {
                    //listening connection
                    tokio::spawn(async {
                        if let Err(err) = connection.await {
                            eprintln!("Connection error: {}", err);
                        }
                    });

                    clients.push_back(client);
                }
                Err(err) => {
                    panic!("Fail to connect to database: {}", err);
                }
            }
        }

        PgPool {
            clients: Arc::new(Mutex::new(clients)),
            semaphore: Arc::new(Semaphore::new(size)),
        }
    }

    pub async fn get_connection(self: &Arc<Self>) -> Result<PgConnection, AcquireError> {
        // Acquire the permit first
        let permit = self.semaphore.clone().acquire_owned().await?;

        // Get a client from the pool
        let client = {
            let mut clients = self.clients.lock().await;
            clients.pop_front().expect("client pool underflow")
        };

        Ok(PgConnection {
            client: Some(client),
            pool: Arc::clone(&self),
            _permit: permit,
        })
    }

    pub async fn return_client(self: &mut Arc<Self>, client: Client) {
        let mut clients = self.clients.lock().await;
        clients.push_back(client);
    }
}

pub struct PgConnection {
    client: Option<Client>,
    pool: Arc<PgPool>,
    _permit: OwnedSemaphorePermit,
}

impl PgConnection {
    pub fn get_client(&self) -> &Client {
        self.client.as_ref().expect("Client was taken")
    }
}

impl Drop for PgConnection {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let mut pool = self.pool.clone();

            tokio::spawn(async move {
                pool.return_client(client).await;
            });
        }
    }
}
