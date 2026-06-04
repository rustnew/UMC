use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_expiry_secs: u64,
    pub jwt_refresh_expiry_secs: u64,
    pub upload_dir: String,
    pub output_dir: String,
    pub max_upload_bytes: u64,
    pub max_concurrent_conversions: usize,
    pub cors_origin: String,
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }
        Ok(())
    }

    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            host:               env::var("UMC_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port:               env::var("UMC_PORT").unwrap_or_else(|_| "8080".into()).parse()?,
            database_url:       env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            jwt_secret:         env::var("JWT_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?,
            jwt_access_expiry_secs:  env::var("JWT_ACCESS_EXPIRY_SECS")
                .unwrap_or_else(|_| "3600".into()).parse()?,
            jwt_refresh_expiry_secs: env::var("JWT_REFRESH_EXPIRY_SECS")
                .unwrap_or_else(|_| "2592000".into()).parse()?,
            upload_dir:         env::var("UPLOAD_DIR").unwrap_or_else(|_| "/tmp/umc/uploads".into()),
            output_dir:         env::var("OUTPUT_DIR").unwrap_or_else(|_| "/tmp/umc/outputs".into()),
            max_upload_bytes:   env::var("MAX_UPLOAD_BYTES")
                .unwrap_or_else(|_| "107374182400".into()).parse()?, // 100 GiB
            max_concurrent_conversions: env::var("MAX_CONCURRENT_CONVERSIONS")
                .unwrap_or_else(|_| "4".into()).parse()?,
            cors_origin:        env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into()),
        })
    }
}
