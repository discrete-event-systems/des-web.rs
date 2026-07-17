//! Env-driven configuration. Follows the k8s-cluster house pattern: HOST/PORT
//! for binding, a Postgres URL resolved from the shared RDS env-name chain, and
//! optional Supabase + des-rs upstream integrations that degrade gracefully
//! when unset (the server always boots and serves pages).

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    /// Resolved Postgres URL. Resolution order mirrors pg-defs tooling:
    /// DES_WEB_DATABASE_URL, DATABASE_URL, SUPABASE_DB_URL, RDS_DATABASE_URL,
    /// PG_DATABASE_URL. `None` runs the server in degraded (no-DB) mode.
    pub database_url: Option<String>,
    /// Supabase project URL (e.g. https://xyz.supabase.co) for GoTrue auth.
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    /// Base URL of a running des-rs (k8s-cluster) instance. When set, the
    /// copied soccer-planner page gets live solves proxied to the real engine.
    pub des_upstream_url: Option<String>,
}

fn env_opt(name: &str) -> Option<String> {
    env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl Config {
    pub fn from_env() -> Config {
        let database_url = env_opt("DES_WEB_DATABASE_URL")
            .or_else(|| env_opt("DATABASE_URL"))
            .or_else(|| env_opt("SUPABASE_DB_URL"))
            .or_else(|| env_opt("RDS_DATABASE_URL"))
            .or_else(|| env_opt("PG_DATABASE_URL"));
        Config {
            host: env_opt("HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
            port: env_opt("PORT").and_then(|p| p.parse().ok()).unwrap_or(8130),
            database_url,
            supabase_url: env_opt("SUPABASE_URL"),
            supabase_anon_key: env_opt("SUPABASE_ANON_KEY"),
            des_upstream_url: env_opt("DES_UPSTREAM_URL")
                .map(|u| u.trim_end_matches('/').to_string()),
        }
    }

    pub fn supabase(&self) -> Option<(&str, &str)> {
        match (self.supabase_url.as_deref(), self.supabase_anon_key.as_deref()) {
            (Some(url), Some(key)) => Some((url, key)),
            _ => None,
        }
    }
}
