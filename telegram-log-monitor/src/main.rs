use std::process::{Command, Stdio};
use std::io::{self, BufRead};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use reqwest::Client;
use tokio::time::sleep;
use urlencoding::encode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(Client::new());
    let last_network_log = Arc::new(Mutex::new(Instant::now()));
    let mut child = Command::new("log")
        .arg("stream")
        .arg("--predicate")
        .arg("process == \"Telegram\"")
        .arg("--level")
        .arg("info")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let reader = io::BufReader::new(stdout);
    let mut lines = reader.lines();

    let client_clone = client.clone();
    let last_clone = last_network_log.clone();
    tokio::spawn(async move {
        loop {
            let now = Instant::now();
            let duration = {
                let guard = last_clone.lock().unwrap();
                now.duration_since(*guard)
            };
            if duration > Duration::from_secs(60) {
                if let Err(e) = send_network_error(&client_clone).await {
                    eprintln!("Failed to send network error: {}", e);
                }
                // Reset to avoid spamming
                *last_clone.lock().unwrap() = Instant::now();
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    while let Some(line) = lines.next() {
        let line = line?;
        if line.contains("Telegram: (UserNotifications)") {
            if let Err(e) = send_notification(&client).await {
                eprintln!("Failed to send notification: {}", e);
            }
        }
        if line.contains("Telegram: (Network)") {
            *last_network_log.lock().unwrap() = Instant::now();
        }
    }

    Ok(())
}

async fn send_notification(client: &Client) -> anyhow::Result<()> {
    let url = "https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=%E5%90%88%E7%BA%A6%E7%BE%A4%E9%87%8C%E6%9C%89%E6%96%B0%E6%B6%88%E6%81%AF%EF%BC%8C%E8%AF%B7%E6%B3%A8%E6%84%8F%EF%BC%81%EF%BC%81%EF%BC%81";
    client.get(url).send().await?;
    Ok(())
}

async fn send_network_error(client: &Client) -> anyhow::Result<()> {
    let text = encode("本地telegram网络错误");
    let url = format!("https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text={}", text);
    client.get(url).send().await?;
    Ok(())
}