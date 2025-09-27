use surrealdb::{
    engine::remote::ws::{Client, Wss}, opt::auth::{Database, Namespace, Root}, Surreal
};

/// Creates and returns a SurrealDB connection
pub async fn setup_surrealdb(
    address: String,
    user: String,
    password: String,
    namespace: String,
    database_name: String,
) -> Result<Surreal<Client>, surrealdb::Error> {
    let db = Surreal::new::<Wss>(address).await?;

    let connect_database_result = db
        .signin(Database {
            namespace: &namespace,
            database: &database_name,
            username: &user,
            password: &password,
        })
        .await;

    match connect_database_result {
        Ok(_) => return Ok(db),
        Err(error) => {
            if !is_authentication_error(&error) {
                return Err(error);
            }
        }
    };

    let connect_namespace_result = db
        .signin(Namespace {
            namespace: &namespace,
            username: &user,
            password: &password,
        })
        .await;

    match connect_namespace_result {
        Ok(_) => {
            db.use_db(database_name).await?;
            return Ok(db);
        }
        Err(error) => if !is_authentication_error(&error) {
            return Err(error);
        }
    };

    db.signin(Root {
        username: &user,
        password: &password,
    })
    .await?;

    db.use_ns(namespace).await?;
    db.use_db(database_name).await?;

    Ok(db)
}

/// Checks whether the error is a connection error (if returned by a signin function)
fn is_authentication_error(error: &surrealdb::Error) -> bool {
    if let surrealdb::Error::Api(error) = error
        && matches!(error, surrealdb::error::Api::Query(_))
    {
        true
    } else {
        false
    }
}
