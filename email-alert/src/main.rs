use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use email_alert::config::Settings;
use email_alert::email::EmailClient;
use tracing::info;

#[get("/send-email")]
async fn send_email_handler(data: web::Data<AppState>) -> impl Responder {
    info!("Received request to send email");

    // 发送邮件
    match data.email_client.send_email(
        "Test Email from Email Alert Service",
        "This is a test email sent from the Email Alert Service.\n\nIf you received this, the service is working correctly!",
    ) {
        Ok(_) => {
            info!("Email sent successfully");
            HttpResponse::Ok().body("Email sent successfully!")
        }
        Err(e) => {
            info!("Failed to send email: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to send email: {:?}", e))
        }
    }
}

struct AppState {
    email_client: EmailClient,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("email_alert=info".parse().unwrap()),
        )
        .init();

    info!("Starting Email Alert Service");

    // 加载配置
    let settings = Settings::load()?;
    info!("Configuration loaded successfully");

    // 初始化邮件客户端
    let email_client = EmailClient::new(settings.smtp.clone());

    // 创建应用状态
    let app_state = web::Data::new(AppState {
        email_client,
    });

    // 启动 HTTP 服务器
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(send_email_handler)
    })
    .bind(format!("0.0.0.0:{}", settings.server.port))?
    .run();

    info!("Server started on port {}", settings.server.port);
    info!("Send GET request to /send-email to trigger email sending");

    server.await?;

    Ok(())
}
