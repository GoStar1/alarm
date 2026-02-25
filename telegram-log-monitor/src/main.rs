use std::process::{Command, Stdio};
use std::io::{self, BufRead};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use reqwest::Client;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(Client::new());
    let last_network_log = Arc::new(Mutex::new(Instant::now()));
    
    // Start network check task
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
    
    // Start log monitoring task
    let client_clone2 = client.clone();
    let last_clone2 = last_network_log.clone();
    tokio::spawn(async move {
        loop {
            match Command::new("log")
                .arg("stream")
                .arg("--predicate")
                .arg("process == \"Telegram\"")
                .arg("--level")
                .arg("info")
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    let stdout = child.stdout.take().expect("Failed to open stdout");
                    let reader = io::BufReader::new(stdout);
                    let mut lines = reader.lines();
                    
                    while let Some(line) = lines.next() {
                        match line {
                            Ok(line) => {
                                if line.contains("Telegram: (UserNotifications)") {
                                    if let Err(e) = send_notification(&client_clone2).await {
                                        eprintln!("Failed to send notification: {}", e);
                                    }
                                }
                                if line.contains("Telegram: (Network)") {
                                    *last_clone2.lock().unwrap() = Instant::now();
                                }
                            },
                            Err(e) => {
                                eprintln!("Error reading log line: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to start log stream: {}", e);
                    eprintln!("Will try again in 10 seconds...");
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    });
    
    // Keep the main thread running
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

async fn send_notification(client: &Client) -> anyhow::Result<()> {
    let url = "https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=New%20group%20notification";
    client.get(url).send().await?;
    Ok(())
}

async fn send_network_error(client: &Client) -> anyhow::Result<()> {
    let url = "https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=Local%20Telegram%20service%20is%20down.";
    client.get(url).send().await?;
    Ok(())
}