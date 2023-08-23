use serde::{Deserialize, Serialize};

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
  pub wit: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DatabasesURIs {
  pub redis: String,
  pub postresql: String,
  pub postresql_dbname: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct DatabaseAuthentificationTokens {
  pub postresql_login: String,
  pub postresql_password: String,
}
