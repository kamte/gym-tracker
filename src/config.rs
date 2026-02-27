use std::env;

const KNOWN_DEFAULT_SECRETS: &[&str] = &[
    "dev-secret-change-in-production-minimum-32-bytes!!",
    "change-this-to-a-random-secret-at-least-32-bytes-long",
    "change-this-to-a-random-secret-at-least-32-bytes-long!!",
];

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        if KNOWN_DEFAULT_SECRETS.contains(&jwt_secret.as_str()) {
            if env::var("ALLOW_DEFAULT_SECRET").is_ok() {
                tracing::warn!("Using a known default JWT_SECRET. Do NOT use this in production!");
            } else {
                panic!(
                    "JWT_SECRET is set to a known default value. This is insecure.\n\
                     Generate a secure secret with: openssl rand -base64 32\n\
                     Set ALLOW_DEFAULT_SECRET=1 to override for local development."
                );
            }
        }

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./gym.db".to_string()),
            jwt_secret,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        }
    }
}
