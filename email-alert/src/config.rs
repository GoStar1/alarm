use dotenvy::var;

#[derive(Debug, Clone)]
pub struct Settings {
    pub smtp: SmtpConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub to_email: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        // 加载 .env 文件
        dotenvy::from_path(".env").ok();

        Ok(Self {
            smtp: SmtpConfig {
                server: var("SMTP_SERVER").unwrap_or_else(|_| "smtp.example.com".to_string()),
                port: var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse()?,
                username: var("SMTP_USERNAME").unwrap_or_else(|_| "your_email@example.com".to_string()),
                password: var("SMTP_PASSWORD").unwrap_or_else(|_| "your_password".to_string()),
                from_email: var("FROM_EMAIL").unwrap_or_else(|_| "your_email@example.com".to_string()),
                to_email: var("TO_EMAIL").unwrap_or_else(|_| "recipient@example.com".to_string()),
            },
            server: ServerConfig {
                port: var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse()?,
            },
        })
    }
}
