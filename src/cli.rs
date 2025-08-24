use crate::auth::AuthService;
use crate::db::BearoData;
use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("your-app")
        .subcommand(
            Command::new("create-admin-key")
                .about("Create a new admin API key")
                .arg(
                    Arg::new("key")
                        .long("key")
                        .help("Custom key (optional, will generate if not provided)")
                        .value_name("KEY"),
                ),
        )
        .subcommand(Command::new("list-admins").about("List all admin API keys"))
        .subcommand(
            Command::new("revoke-key").about("Revoke an API key").arg(
                Arg::new("key")
                    .long("key")
                    .help("The API key to revoke")
                    .value_name("KEY")
                    .required(true),
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
                    println!("admin API key created successfully!");
                    println!("Key: {}", key);
                    println!("ID: {}", api_key.oid);
                    println!("Created at: {}", api_key.created_at);

                    let all_keys = auth_service.list_api_keys(&db).await?;
                    let admin_count = all_keys.iter().filter(|k| k.is_admin).count();
                    println!("Total admin keys: {}", admin_count);
                }
                Err(e) => {
                    eprintln!("failed to create admin key: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("list-admins", _)) => {
            let db = create_db_connection().await?;
            let all_keys = auth_service.list_api_keys(&db).await?;
            let admin_keys: Vec<_> = all_keys.iter().filter(|k| k.is_admin).collect();

            if admin_keys.is_empty() {
                println!("no admin keys found.");
            } else {
                println!("Admin API Keys ({} total):", admin_keys.len());
                println!("{:<25} {:<20} {:<20}", "ID", "Created At", "Last Used");
                println!("{}", "-".repeat(65));

                for key in admin_keys {
                    let last_used = key
                        .last_used_at
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Never".to_string());

                    println!(
                        "{:<25} {:<20} {:<20}",
                        key.oid.to_hex(),
                        key.created_at.format("%Y-%m-%d %H:%M:%S"),
                        last_used
                    );
                }
            }
        }
        Some(("revoke-key", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap();
            let db = create_db_connection().await?;

            match auth_service.revoke_api_key(key, &db).await {
                Ok(()) => {
                    println!("api key revoked successfully!");

                    let all_keys = auth_service.list_api_keys(&db).await?;
                    let admin_count = all_keys.iter().filter(|k| k.is_admin).count();

                    if admin_count == 0 {
                        println!(
                            "[WARN]: No admin keys remain! You may need to use BOOTSTRAP_ADMIN_KEY."
                        );
                    } else {
                        println!("remaining admin keys: {}", admin_count);
                    }
                }
                Err(e) => {
                    eprintln!("failed to revoke key: {}", e);
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
