use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use std::time;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub async fn create_connection_pool(
    connection_string: &str,
    number_connections: u32,
) -> anyhow::Result<Pool<ConnectionManager>> {
    let connection = ConnectionManager::build(connection_string)?;

    // The following code does not connect to the SQL Server instance
    // until you call the `get()` function from the `Pool<ConnectionManager>`.
    let pool = bb8::Pool::builder()
        .max_size(number_connections)
        .connection_timeout(time::Duration::from_secs(300))
        .build(connection)
        .await?;

    Ok(pool)
}

pub async fn get_tables(pool:Pool<ConnectionManager>)->anyhow::Result<Vec<String>>{
    let mut connection=pool.get().await?;

    let tables=connection.simple_query("select name from sys.tables")
        .await?
        .into_first_result().await?
        .into_iter()
        .map(|row|{
            let name:&str=row.get(0).unwrap();
            String::from(name)
        })
        .collect::<Vec<String>>();

    Ok(tables)
}

pub async fn register_player_with_dedicated_connection(
    connection_string: &'static str,
    number_clients: i32,
) -> anyhow::Result<()> {
    let mut handles: Vec<JoinHandle<i32>> = vec![];

    // Simulate several clients connecting to SQL Server
    for i in 1..=number_clients {
        let handle = tokio::spawn(async move {
            let config = Config::from_ado_string(connection_string).unwrap();
            let tcp = TcpStream::connect(config.get_addr()).await.unwrap();

            let mut connection = Client::connect(config, tcp.compat_write()).await.unwrap();
            let _ = connection
                .execute(
                    "insert into dbo.Player(FirstName,LastName,Phone)\
            values(@P1,@P2,@P3)",
                    &[
                        &format!("Revenant {}", i),
                        &"Buttskicker",
                        &(1_000_000 + i),
                    ],
                )
                .await
                .unwrap();

            let _ = connection.close().await;
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        match handle.await {
            Ok(h) => println!("Player {} registered.", h),
            Err(e) => panic!("Failed to register player {}.", e),
        }
    }

    Ok(())
}

pub async fn register_player_with_connection_pool(
    pool: Pool<ConnectionManager>,
    number_clients: i32,
) -> anyhow::Result<()> {
    let mut handles: Vec<JoinHandle<i32>> = vec![];

    // Simulate clients connecting to SQL Server
    for i in 1..=number_clients {
        let pool = pool.clone();

        let handle = tokio::spawn(async move {
            let mut connection = pool.get().await.unwrap();
            let _ = connection
                .execute(
                    "insert into dbo.Player(FirstName,LastName,Phone)\
            values(@P1,@P2,@P3)",
                    &[
                        &format!("Revenant {}", i),
                        &"Buttskicker",
                        &(1_000_000 + i),
                    ],
                )
                .await
                .unwrap();
            i
        });
        handles.push(handle);
    }

    // Wait for all connections to complete
    for handle in handles {
        match handle.await {
            Ok(h) => println!("Client {} registered.", h),
            Err(e) => panic!("Failed to register client {}.", e),
        }
    }

    Ok(())
}