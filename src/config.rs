use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
  pub tokens: Tokens,
  pub databases: DatabasesURIs,
  pub auth: DatabaseAuthentificationTokens,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Tokens {
  pub discord: String,
  pub google: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DatabasesURIs {
  pub redis: String,
  pub surrealdb: String
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DatabaseAuthentificationTokens {
  pub surrealdb_login: String,
  pub surrealdb_password: String,
}