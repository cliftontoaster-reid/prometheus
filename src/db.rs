use redis::{aio::Connection, AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::Message, prelude::Context};
use tokio_postgres::{Client as PostgreSQLClient, NoTls};

use crate::discord::DiscordClient;

pub struct Client {
  pub postgresql: PostgreSQLClient,
  pub redis: Connection,
  pub redis_base: RedisClient,
}

pub enum EntryID {
  Base(String),
  WithID((String, u64)),
}

impl Client {
  pub async fn connect(
    postgre: String,
    auth: (String, String),
    db_name: String,
    redis: String,
  ) -> Self {
    let (postgresql, connection) = tokio_postgres::connect(
      &format!(
        "host={} user={} password={} dbname={}",
        postgre, auth.0, auth.1, db_name
      ),
      NoTls,
    )
      .await
      .unwrap();

    tokio::spawn(async move {
      if let Err(e) = connection.await {
        eprintln!("connection error: {}", e);
      }
    });

    let redis = RedisClient::open(redis).unwrap();

    let s = Self {
      postgresql,
      redis: redis.get_async_connection().await.unwrap(),
      redis_base: redis.clone(),
    };
    s
  }

  async fn init_db(&self) {
    // Server
    self
      .postgresql
      .query(
        "
      CREATE TABLE IF NOT EXISTS servers (
        id bigint UNIQUE PRIMARY KEY,
        beta_program boolean NOT NULL
      );
    ",
        &[],
      )
      .await
      .unwrap();

    self
      .postgresql
      .query(
        "
      CREATE TABLE IF NOT EXISTS welcome (
        server_id bigint UNIQUE PRIMARY KEY,
        channel_id bigint NOT NULL
      );
    ",
        &[],
      )
      .await
      .unwrap();
  }

  pub async fn get_server(&self, server_id: u64) -> Option<ServerConfig> {
    let mut con = self.redis_base.get_async_connection().await.unwrap();
    let cached: Option<String> = con
      .get(format!("server_{}", server_id.to_string()))
      .await
      .unwrap();

    match cached {
      Some(cache) => toml::from_str(&cache).unwrap(),
      None => {
        let res = self
          .postgresql
          .query(
            "SELECT * FROM servers WHERE id = $1;",
            &[&(server_id as i64)],
          )
          .await
          .unwrap();
        if res.is_empty() {
          None
        } else {
          let row = res.get(0).unwrap();
          if row.is_empty() {
            return None;
          }
          let id: i64 = row.get(0);
          return Some(ServerConfig {
            id: id as u64,
            beta_program: row.get(1),
          });
        }
      }
    }
  } // Done

  pub async fn update_server(&self, server: ServerConfig) {
    todo!()
  }

  pub async fn create_server(&self, server: ServerConfig) {
    let s: String = toml::to_string(&server).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _: () = redis::pipe()
      .cmd("SET")
      .arg(format!("server_{}", &server.id.to_string()))
      .arg(s)
      .ignore()
      .cmd("EXPIRE")
      .arg(format!("server_{}", &server.id.to_string()))
      .arg(120)
      .ignore()
      .query(&mut con)
      .unwrap();

    self
      .postgresql
      .query(
        "
      INSERT INTO servers(id, beta_program)
      VALUES ($1, $2);
    ",
        &[&(server.id as i64), &server.beta_program],
      )
      .await
      .unwrap();
  } // Done

  pub async fn get_welcome(&self, server_id: u64) -> Option<WelcomeConfig> {
    let mut con = self.redis_base.get_async_connection().await.unwrap();
    let cached: Option<String> = con
      .get(format!("welcome_{}", server_id.to_string()))
      .await
      .unwrap();

    match cached {
      Some(cache) => Some(toml::from_str(&cache).unwrap()),
      None => {
        let res = self
          .postgresql
          .query(
            "SELECT * FROM welcome WHERE server_id = $1;",
            &[&(server_id as i64)],
          )
          .await
          .unwrap();
        if res.is_empty() {
          None
        } else {
          let row = res.get(0).unwrap();
          if row.is_empty() {
            return None;
          }
          let server: i64 = row.get(0);
          let channel: i64 = row.get(1);
          let option = Some(WelcomeConfig {
            server_id: server as u64,
            channel_id: channel as u64,
          });
          return option;
        }
      }
    }
  } // Done

  pub async fn update_welcome(&self, config: WelcomeConfig, server_id: u64) {
    todo!()
  }

  pub async fn create_welcome(&self, config: WelcomeConfig, server_id: u64) {
    let s: String = toml::to_string(&config).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _: () = redis::pipe()
      .cmd("SET")
      .arg(format!("welcome_{}", server_id.to_string()))
      .arg(s)
      .ignore()
      .cmd("EXPIRE")
      .arg(format!("welcome_{}", server_id.to_string()))
      .arg(120)
      .ignore()
      .query(&mut con)
      .unwrap();

    self
      .postgresql
      .query(
        "
      INSERT INTO welcome(server_id, channel_id)
      VALUES ($1, $2);
    ",
        &[&(config.server_id as i64), &(config.channel_id as i64)],
      )
      .await
      .unwrap();
  } // Done

  pub async fn delete_welcome(&self, server_id: u64) {
    let mut con = self.redis_base.get_connection().unwrap();
    let _: () = redis::pipe()
      .cmd("DEL")
      .arg(format!("welcome_{}", server_id.to_string()))
      .ignore()
      .query(&mut con)
      .unwrap();

    self
      .postgresql
      .query(
        "
        DELETE FROM welcome
        WHERE server_id = $1;
      ",
        &[&(server_id as i64)],
      )
      .await
      .unwrap();
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerConfig {
  pub id: u64,
  pub beta_program: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Features {
  Welcome(WelcomeConfig),
}

impl Features {
  pub fn enable_successful(&self) -> String {
    format!("My distinguished lord,

I humbly bring to your attention tidings of great import. The very feature thou didst beseech, namely {}, hath been ushered into existence within this realm. May I express my utmost delight in relaying this news, and I beseech thee to partake in its offerings with the same zeal that ignited its conception. In unwavering reverence, I remain...
    
Yours in loyal service,
    
Clifton Toaster Reid
Bearer of the Prometheus Banner", match self {
      Features::Welcome(_) => "the welcome messages",
    })
  }
  pub fn disable_successful(&self) -> String {
    format!("Honorable Sir,

May it please thee to know that the feature thou didst inquire about, {} by name, hath been deemed unfit for this realm and thus remaineth disabled. It is with regret that I bear this message, for it is my utmost desire to fulfill thy wishes. Thy understanding and continued guidance are invaluable to us.

With sincerest regards,

Clifton Toaster Reid
Bearer of the Prometheus Banner", match self {
      Features::Welcome(_) => "the welcome messages",
    })
  }

  pub async fn enable_str(s: &DiscordClient, msg: &Message, ctx: &Context, intent: &String) {
    if !msg
      .member(&ctx)
      .await
      .unwrap()
      .permissions(&ctx)
      .unwrap()
      .manage_guild()
    {
      msg.reply_mention(&ctx, "Kind and honorable Sir,

With sincere regret, I must impart that the permissions requisite for the action thou wishest to undertake have not been bestowed upon thee at this juncture. Worry not, for the potential lies ahead. When thou art granted the authority befitting thy station, the path shall be clear for thee to partake in the endeavor. Until then, I remain at your service to address any queries or concerns.
      
With the utmost respect,

Clifton Toaster Reid
Bearer of the Prometheus Banner").await.unwrap();
      return;
    }

    let feature = intent.strip_suffix("_enable").unwrap();
    let repl = match feature {
      "welcome" => {
        let channel = if msg.mention_channels.is_empty() {
          msg.channel_id
        } else {
          msg.mention_channels.get(0).unwrap().id
        };
        let t = WelcomeConfig::new(msg.guild_id.unwrap().0, channel.0, &s.db).await;
        Features::enable_successful(&Features::Welcome(t))
      }
      _ => "Most wondrous and esteemed Sir,

In awe do I pen these words, for thou hast proven to transcend the passage of ages. How thou came to know of a feature ere its birth baffles my comprehension, yet it is with the utmost admiration that I extend my heartfelt felicitations unto thee. A true master of temporal boundaries, a challenger of time itself, thou art.
      
With boundless respect and marvel,
      
Clifton Toaster Reid
Bearer of the Prometheus Banner".to_owned(),
    };

    msg.reply(&ctx, repl).await.unwrap();
  }

  pub async fn disable_str(s: &DiscordClient, msg: &Message, ctx: &Context, intent: &String) {
    if !msg
      .member(&ctx)
      .await
      .unwrap()
      .permissions(&ctx)
      .unwrap()
      .manage_guild()
    {
      msg.reply_mention(&ctx, "Kind and honorable Sir,

With sincere regret, I must impart that the permissions requisite for the action thou wishest to undertake have not been bestowed upon thee at this juncture. Worry not, for the potential lies ahead. When thou art granted the authority befitting thy station, the path shall be clear for thee to partake in the endeavor. Until then, I remain at your service to address any queries or concerns.
      
With the utmost respect,

Clifton Toaster Reid
Bearer of the Prometheus Banner").await.unwrap();
      return;
    }

    let feature = intent.strip_suffix("_disable").unwrap();
    let repl = match feature {
      "welcome" => {
        match s.db.get_welcome(msg.guild_id.unwrap().0).await {
          Some(uwu) => {
            s.db.delete_welcome(msg.guild_id.unwrap().0).await;
            Features::enable_successful(&Features::Welcome(uwu))
          },
          None => {
            Features::enable_successful(&Features::Welcome(WelcomeConfig { server_id: msg.guild_id.unwrap().0, channel_id: msg.channel_id.0 }))
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WelcomeConfig {
  pub server_id: u64,
  pub channel_id: u64,
}

impl WelcomeConfig {
  pub async fn new(server_id: u64, channel_id: u64, db: &Client) -> WelcomeConfig {
    let server = Self {
      server_id,
      channel_id,
    };
    db.create_welcome(server, server_id).await;
    server
  }
}
