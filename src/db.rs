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

  pub async fn get_server(&mut self, server_id: u64) -> Option<ServerConfig> {
    let cached: Option<String> = self.redis.get(format!("server_{}", server_id)).await.unwrap();
    match cached {
      Some(cache) => toml::from_str(&cache).unwrap_or(self.surreal.select(("server", server_id)).await.unwrap()),
      None => self.surreal.select(("server", server_id)).await.unwrap(),
    }
  }

  pub async fn update_server(&mut self, server: ServerConfig) -> Option<ServerConfig> {
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
  pub id: u64,
  pub beta_program: bool,
}