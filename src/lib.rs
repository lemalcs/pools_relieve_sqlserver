mod pools;

use pools::*;

#[tokio::test]
async fn test_get_tables() {
    let pool=create_connection_pool(
        "server=tcp:127.0.0.1,22828;database=master;trustServerCertificate=true;IntegratedSecurity=true",
        2).await.unwrap();

    let result = get_tables(pool).await;

    dbg!(&result);

    assert!(result.unwrap().len() > 0, "Query executed successfully")
}

#[tokio::test]
async fn test_register_player_with_dedicated_connection() {
    let number_clients=300_000;
    let result= register_player_with_dedicated_connection(
        "server=tcp:127.0.0.1,22828;database=Trailblazer;trustServerCertificate=true;IntegratedSecurity=true;Connect Timeout=0",
        number_clients).await;

    assert!(result.is_ok(), "Query executed successfully")
}

#[tokio::test]
async fn test_registering_player_with_connection_pool() {
    let number_clients=300_000;
    let pool=create_connection_pool(
        "server=tcp:127.0.0.1,22828;database=Trailblazer;trustServerCertificate=true;IntegratedSecurity=true",
        2).await.unwrap();

    let result = register_player_with_connection_pool(pool, number_clients).await;

    assert!(result.is_ok(), "Query executed successfully")
}