use crate::config::SmtpConfig;
use crate::error::AppError;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct EmailClient {
    config: SmtpConfig,
}

impl EmailClient {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    pub fn send_email(&self, subject: &str, body: &str) -> Result<(), AppError> {
        // 创建邮件消息
        let email = Message::builder()
            .from(self.config.from_email.parse().map_err(|e: lettre::address::AddressError| AppError::EmailError(e.to_string()))?)
            .to(self.config.to_email.parse().map_err(|e: lettre::address::AddressError| AppError::EmailError(e.to_string()))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| AppError::EmailError(e.to_string()))?;

        // 设置 SMTP 传输
        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());

        let mailer = SmtpTransport::relay(&self.config.server)
            .map_err(|e| AppError::EmailError(e.to_string()))?
            .port(self.config.port)
            .credentials(creds)
            .build();

        // 发送邮件
        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError::EmailError(e.to_string())),
        }
    }
}
