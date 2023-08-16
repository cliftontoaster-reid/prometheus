use crate::db::{Client as DBClient, ServerConfig};
use serenity::{
  async_trait,
  model::{channel::Message, gateway::Ready},
  prelude::*,
};

use crate::safety::discord::check_malicious;

pub struct DiscordClient {
  pub db: DBClient,

  pub google_key: String,
}

#[async_trait]
impl EventHandler for DiscordClient {
  async fn message(&self, ctx: Context, msg: Message) {
    let serv: ServerConfig = match self.db.get_server(msg.author.id).await {
      Some(server) => server,
      None => self
        .db
        .create_server(ServerConfig {
          id: msg.author.id.0,
          beta_program: false,
        })
        .await
        .unwrap(),
    };
    
    if !msg.content.starts_with(".")
      | msg.author.bot
      | check_malicious(&self.google_key, &ctx, &msg).await
    {
      return;
    }
  }
  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
  }
}
