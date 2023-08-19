use serde::{Deserialize, Serialize};
use redis::{Client as RedisClient, aio::Connection, AsyncCommands};
use surrealdb::{
  engine::remote::ws::{Client as WsClient, Ws},
  opt::auth::Root,
  Surreal,
};


pub struct Client {
  pub surreal: Surreal<WsClient>,
  pub redis: Connection,
  pub redis_base: RedisClient,
}

impl Client {
  pub async fn connect(surreal: String, auth: (String, String), redis: String) -> Self {
    let surreal = Surreal::new::<Ws>(surreal).await.unwrap();

    surreal.signin(Root {
      username: &auth.0,
      password: &auth.1,
    })
    .await
    .unwrap();
    surreal.use_ns("toast_n_co").use_db("prometheus").await.unwrap();

    let redis = RedisClient::open(redis).unwrap();

    Self { surreal, redis: redis.get_async_connection().await.unwrap(), redis_base: redis.clone() }
  }

  pub async fn get_server(&self, server_id: u64) -> Option<ServerConfig> {
    let mut con = self.redis_base.get_async_connection().await.unwrap();
    let cached: Option<String> = con.get(format!("server_{}", server_id)).await.unwrap();
    match cached {
      Some(cache) => toml::from_str(&cache).unwrap_or(self.surreal.select(("server", server_id)).await.unwrap()),
      None => self.surreal.select(("server", server_id)).await.unwrap(),
    }
  }

  pub async fn update_server(&self, server: ServerConfig) -> Option<ServerConfig> {
    let s: String = toml::to_string(&server).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _ : () = redis::pipe()
      .cmd("SET").arg(format!("server_{}", server.id)).arg(s).ignore()
      .cmd("EXPIRE").arg(format!("server_{}", server.id)).arg(120).ignore()
      .query(&mut con).unwrap();

    self.surreal.update(("server", server.id)).content(server).await.unwrap()
  }

  pub async fn create_server(&self, server: ServerConfig) -> Option<ServerConfig> {
    let s: String = toml::to_string(&server).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _ : () = redis::pipe()
      .cmd("SET").arg(format!("server_{}", server.id)).arg(s).ignore()
      .cmd("EXPIRE").arg(format!("server_{}", server.id)).arg(120).ignore()
      .query(&mut con).unwrap();

    self.surreal.create(("server", server.id)).content(server).await.unwrap()
  }

  pub async fn get_welcome(&self, server_id: u64) -> Option<WelcomeConfig> {
    let mut con = self.redis_base.get_async_connection().await.unwrap();
    let cached: Option<String> = con.get(format!("welcome_{}", server_id)).await.unwrap();
    match cached {
      Some(cache) => toml::from_str(&cache).unwrap_or(self.surreal.select(("server", server_id)).await.unwrap()),
      None => self.surreal.select(("welcome", server_id)).await.unwrap(),
    }
  }

  pub async fn update_welcome(&self, config: WelcomeConfig) -> Option<WelcomeConfig> {
    let s: String = toml::to_string(&config).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _ : () = redis::pipe()
      .cmd("SET").arg(format!("welcome_{}", config.server_id)).arg(s).ignore()
      .cmd("EXPIRE").arg(format!("welcome_{}", config.server_id)).arg(120).ignore()
      .query(&mut con).unwrap();

    self.surreal.update(("welcome", config.server_id)).content(config).await.unwrap()
  }

  pub async fn create_welcome(&self, config: WelcomeConfig) -> Option<WelcomeConfig> {
    let s: String = toml::to_string(&config).unwrap();

    let mut con = self.redis_base.get_connection().unwrap();
    let _ : () = redis::pipe()
      .cmd("SET").arg(format!("welcome_{}", config.server_id)).arg(s).ignore()
      .cmd("EXPIRE").arg(format!("welcome_{}", config.server_id)).arg(120).ignore()
      .query(&mut con).unwrap();

    self.surreal.create(("welcome", config.server_id)).content(config).await.unwrap()
  }

  pub async fn delete_welcome(&self, server_id: u64) {
    let mut con = self.redis_base.get_connection().unwrap();
    let _ : () = redis::pipe()
      .cmd("DEL").arg(format!("welcome_{}", server_id)).ignore()
      .query(&mut con).unwrap();

    self.surreal.delete(("welcome", server_id)).await.unwrap()
  }
  
  pub async fn create_feature(&self, feature: Features) -> Option<Features> {
    match feature {
      Features::Welcome(f) => {
        Some(Features::Welcome(self.create_welcome(f).await.unwrap()))
      },
    }
  }

  pub async fn delete_feature(&self, feature: Features) {
    match feature {
      Features::Welcome(f) => {
        self.delete_welcome(f.server_id).await;
      },
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
  pub id: u64,
  pub beta_program: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Features {
  Welcome(WelcomeConfig),
}

impl Features {
  pub fn enable_successfull(&self) -> String {
    format!("My distinguished lord,

I humbly bring to your attention tidings of great import. The very feature thou didst beseech, namely {}, hath been ushered into existence within this realm. May I express my utmost delight in relaying this news, and I beseech thee to partake in its offerings with the same zeal that ignited its conception. In unwavering reverence, I remain...
    
Yours in loyal service,
    
Clifton Toaster Reid
Bearer of the Prometheus Banner", match self {
      Features::Welcome(_) => "the welcome messages",
    })
  }
  pub fn disable_successfull(&self) -> String {
    format!("Honorable Sir,

May it please thee to know that the feature thou didst inquire about, {} by name, hath been deemed unfit for this realm and thus remaineth disabled. It is with regret that I bear this message, for it is my utmost desire to fulfill thy wishes. Thy understanding and continued guidance are invaluable to us.
    
With sincerest regards,
    
Clifton Toaster Reid
Bearer of the Prometheus Banner", match self {
      Features::Welcome(_) => "the welcome messages",
    })
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WelcomeConfig {
  pub server_id: u64,
  pub channel_id: u64,
}

impl WelcomeConfig {
  pub async fn new(server_id: u64, channel_id: u64, db: &Client) -> Option<WelcomeConfig> {
    db.create_welcome(Self {
      server_id,
      channel_id,
    }).await
  }
}