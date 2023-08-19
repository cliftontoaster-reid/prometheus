use crate::wit::Client as NLUClient;
use crate::{
  art::welcome,
  db::{Client as DBClient, Features, ServerConfig, WelcomeConfig},
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

    let meaning = self
      .wit
      .message(
        msg.content.strip_prefix(".").unwrap().trim().to_owned(),
        None,
      )
      .await
      .unwrap();
    if meaning.intents.len() == 0 {
      return;
    }
    let meaning_intent = meaning.intents.get(0).unwrap().name.clone();

    if meaning_intent.ends_with("-enable") {
      let feature = meaning_intent.strip_suffix("-enable").unwrap();
      let repl = match feature {
        "welcome" => {
          let channel = if msg.mention_channels.is_empty() {
            msg.channel_id
          } else {
            msg.mention_channels.get(0).unwrap().id
          };
          let t = WelcomeConfig::new(msg.guild_id.unwrap().0, channel.0, &self.db).await.unwrap();
          Features::enable_successfull(&Features::Welcome(t))
        }
        _ => "Most wondrous and esteemed Sir,

In awe do I pen these words, for thou hast proven to transcend the passage of ages. How thou came to know of a feature ere its birth baffles my comprehension, yet it is with the utmost admiration that I extend my heartfelt felicitations unto thee. A true master of temporal boundaries, a challenger of time itself, thou art.
        
With boundless respect and marvel,
        
Clifton Toaster Reid
Bearer of the Prometheus Banner".to_owned(),
      };

      msg.reply(&ctx, repl).await.unwrap();
    } else if meaning_intent.ends_with("-disable") {
      let feature = meaning_intent.strip_suffix("-enable").unwrap();
      let repl = match feature {
        "welcome" => {
          match self.db.get_welcome(msg.guild_id.unwrap().0).await {
            Some(uwu) => {
              self.db.delete_welcome(msg.guild_id.unwrap().0).await;
              Features::enable_successfull(&Features::Welcome(uwu))
            },
            None => {
              Features::enable_successfull(&Features::Welcome(WelcomeConfig { server_id: msg.guild_id.unwrap().0, channel_id: msg.channel_id.0 }))
            },
          }
        }
        _ => "Most wondrous and esteemed Sir,

In awe do I pen these words, for thou hast proven to transcend the passage of ages. How thou came to know of a feature ere its birth baffles my comprehension, yet it is with the utmost admiration that I extend my heartfelt felicitations unto thee. A true master of temporal boundaries, a challenger of time itself, thou art.
        
With boundless respect and marvel,
        
Clifton Toaster Reid
Bearer of the Prometheus Banner".to_owned(),
      };

      msg.reply(&ctx, repl).await.unwrap();
    }
  }
  async fn ready(&self, _: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
  }
  async fn guild_member_addition(&self, ctx: Context, member: Member) {
    let server = member.guild_id;

    match self.db.get_welcome(server.0).await {
      Some(conf) => {
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
      None => {}
    };
  }
}
