use std::env::args;

use crate::wit::Client as NLUClient;
use crate::{
  art::welcome,
  db::{Client as DBClient, Features, ServerConfig},
};
use serenity::{
  async_trait,
  model::{channel::Message, gateway::Ready, guild::Member, prelude::ChannelId},
  prelude::*,
};
use sha256::digest;
use tokio::fs;

use crate::safety::discord::check_malicious;

pub struct DiscordClient {
  pub db: DBClient,
  pub google_key: String,
  pub wit: NLUClient,
}

#[async_trait]
impl EventHandler for DiscordClient {
  async fn message(&self, ctx: Context, msg: Message) {
    let _serv: ServerConfig = match self.db.get_server(msg.author.id.0).await {
      Some(server) => server,
      None => {
        let server = ServerConfig {
          id: msg.author.id.0,
          beta_program: false,
        };
        self.db.create_server(server).await;
        server
      }
    };

    debug_commands(&self, &msg.clone(), &ctx.clone()).await;

    if !msg.content.starts_with(".")
      | msg.author.bot
      | check_malicious(&self.google_key, &ctx, &msg).await
    {
      return;
    }

    let meaning = self
      .wit
      .message(
        msg.content.strip_prefix(".").unwrap().trim().to_owned(),
        None,
      )
      .await;
    if meaning.intents.len() == 0 {
      return;
    }
    let meaning_intent = meaning.intents.get(0).unwrap().name.clone();

    if meaning_intent.ends_with("_enable") {
      Features::enable_str(&self, &msg, &ctx, &meaning_intent).await
    } else if meaning_intent.ends_with("_disable") {
      Features::disable_str(&self, &msg, &ctx, &meaning_intent).await
    }
  }
  async fn ready(&self, _: Context, ready: Ready) {
    if cfg!(debug_assertions) {
      println!("THIS IS A DEBUG BUILD DO NOT USE IN PRODUCTION");
    }
    println!(
      "{} is connected with session {}!",
      ready.user.name, ready.session_id
    );
  }

  async fn guild_member_addition(&self, ctx: Context, member: Member) {
    println!(
      "{} joined {}",
      member.display_name(),
      member
        .guild_id
        .name(&ctx)
        .unwrap_or("wonderland".to_string())
    );

    let server = member.guild_id;

    match self.db.get_welcome(server.0).await {
      Some(conf) => {
        println!(
          "{} will receive an image on {}",
          member.display_name(),
          member
            .guild_id
            .name(&ctx)
            .unwrap_or("wonderland".to_string())
        );

        let w = welcome(
          member.display_name().as_str(),
          &server.name(&ctx).unwrap_or("wonderland".to_string()),
          &member.avatar_url().unwrap_or(member.user.face()),
        )
        .await;
        let mut path = dirs::cache_dir().unwrap();

        path.push(format!(
          "{}.png",
          digest(format!("{}{}", member.user.id, server.0))
        ));
        w.save(&path).unwrap();

        println!(
          "File created: {}",
          &path.to_str().unwrap_or(&path.to_string_lossy())
        );

        let _ = ChannelId::from(conf.channel_id)
          .send_files(
            &ctx,
            vec![path.to_str().unwrap_or(&path.to_string_lossy())],
            |m| m.content("Welcome!!"),
          )
          .await;

        fs::remove_file(path).await.unwrap();
      }
      None => {
        println!(
          "welcome images are not enabled on {} and so {} won't receive an image",
          member
            .guild_id
            .name(&ctx)
            .unwrap_or("wonderland".to_string()),
          member.display_name()
        );
      }
    };
  }
}

async fn debug_commands(s: &DiscordClient, msg: &Message, ctx: &Context) {
  if msg.content.trim().starts_with("?//d ") {
    let va: Vec<String> = args().collect();
    if !va.contains(&"--dev--".to_string()) {
      println!(
        "{} ID {} tried to use a debug command!!!",
        msg.author.name, msg.author.id.0
      );
      return;
    }
    println!("{}", msg.content.trim());
    let comm: &str = msg.content.strip_prefix("?//d ").unwrap().trim();

    match comm {
      "welcome" => {
        msg.reply(&ctx, "Sending welcome event now!").await.unwrap();
        let m = msg.member(&ctx).await.unwrap();
        s.guild_member_addition(ctx.clone(), m).await
      }
      _ => {
        msg.reply(&ctx, "Invalid debug command!").await.unwrap();
      }
    }
  }
}
