pub mod config;
pub mod db;
pub mod discord;
pub mod safety;
pub mod utils;
pub mod wit;
pub mod art;

use std::process;

use config::Config;
use dirs::config_dir;
use tokio::fs::{create_dir, read_to_string, write};
use toml::{from_str, to_string_pretty};
use discord::DiscordClient;
use db::Client as DBClient;
use serenity::{prelude::Client as SerenityClient, model::gateway::GatewayIntents};
use wit::Client as NLUClient;

#[tokio::main]
async fn main() {
  let mut config_file_path = config_dir().unwrap();
  config_file_path.push("toast_n_co");
  if !config_file_path.exists() {
    create_dir(&config_file_path).await.unwrap()
  }
  config_file_path.push("prometheus");
  if !config_file_path.exists() {
    create_dir(&config_file_path).await.unwrap()
  }
  config_file_path.push("config.toml");
  let config: Config = if !config_file_path.exists() {
    write(
      &config_file_path,
      to_string_pretty(&Config::default()).unwrap(),
    )
    .await
    .unwrap();
    eprintln!(
      "Oh maeany pawsome moments! A totally fur-tastic config file has been awesomely crafted at {}. UwU It's like, time to spread those fluffy wings and soar to the magical land of settings, and like, sprinkle it with your pawsitively correct info! OwO *nuzzles*",
      config_file_path.display().to_string()
    );
    process::exit(1);
  } else {
    let config: Config = from_str(&match read_to_string(config_file_path).await {
      Ok(s) => s,
      Err(_) => unreachable!("Oh noesies! *hides paws* I, like, totally can't, you know, read this file, 'cause, uhm, the super-duper sad thing is that this file, like, might, just might be all, like, corrupted and stuff? *whimpers cutely* OwO *wags tail*"),
    })
    .unwrap();
    if config != Config::default() {
      config
    } else {
      eprintln!("Like, heya there, fluffster! UwU, could you, like, pretty pwease use the super-duper config file, owo? It's like, totes important, and things might get a wittle wonky without it, teehee~!");
      process::exit(1)
    }
  };

  let database_client = DBClient::connect(config.databases.surrealdb, (config.auth.surrealdb_login, config.auth.surrealdb_password), config.databases.redis).await;
  
  let client = DiscordClient {db:database_client,google_key:config.tokens.google, wit: NLUClient{ token: config.tokens.wit } };

  let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

  let mut ser_client = SerenityClient::builder(config.tokens.discord, intents).event_handler(client).await.unwrap();
  ser_client.start_autosharded().await.unwrap();
}
