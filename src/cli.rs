use crate::auth::AuthService;
use crate::db::BearoData;
use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("your-app").subcommand(
        Command::new("create-admin-key")
            .about("Create an initial admin API key")
            .arg(
                Arg::new("key")
                    .long("key")
                    .help("Custom key (optional, will generate if not provided)")
                    .value_name("KEY"),
            ),
    )
}

pub async fn handle_cli(auth_service: AuthService) -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("create-admin-key", sub_matches)) => {
            let key = match sub_matches.get_one::<String>("key") {
                Some(custom_key) => custom_key.clone(),
                None => AuthService::generate_api_key(),
            };

            let db = create_db_connection().await?;

            match auth_service.create_api_key(&key, true, &db).await {
                Ok(api_key) => {
                    println!("API key created successfully!");
                    println!("Key: {}", key);
                    println!("ID: {}", api_key.oid);
                    println!("Created at: {}", api_key.created_at);
                }
                Err(e) => {
                    eprintln!("Failed to create new admin key: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            cli().print_help()?;
        }
    }

    Ok(())
}

async fn create_db_connection() -> Result<BearoData, Box<dyn std::error::Error>> {
    use rocket_db_pools::mongodb::{Client, options::ClientOptions};
    use std::env;

    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("MONGODB_URL"))
        .expect("DATABASE_URL or MONGODB_URL must be set");

    let client_options = ClientOptions::parse(&database_url).await?;
    let client = Client::with_options(client_options)?;

    Ok(BearoData::from(client))
}
