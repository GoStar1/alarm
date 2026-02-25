use std::process::Command;
use std::time::Duration;
use std::sync::Arc;
use reqwest::Client;
use tokio::time::{sleep, timeout};

fn get_telegram_badge() -> Option<String> {

    // 构建 AppleScript 脚本
    let script = r#"
        try
            -- 获取原始信息
            set rawInfo to do shell script "lsappinfo info -only StatusLabel Telegram"
            
            -- 使用更健壮的正则：寻找 label 后面跟着的数字或内容
            -- 如果匹配不到数字，则说明没有未读消息
            if rawInfo contains "\"label\"=" then
                set badgeValue to do shell script "echo " & quoted form of rawInfo & " | sed -E 's/.*\"label\"=\"?([^\"]*)\"?.*/\\1/'"
                
                -- 二次检查：防止提取出 kCFNULL 字符
                if badgeValue contains "kCFNULL" or badgeValue is "" then
                    return ""
                else
                    return badgeValue
                end if
            else
                return ""
            end if
        on error
            return ""
        end try
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(Client::new());
    
    // Start network check task
    let client_clone = client.clone();
    tokio::spawn(async move {
        loop {
            match timeout(Duration::from_secs(10), client_clone.get("https://web.telegram.org/").send()).await {
                Ok(Ok(_)) => {
                    println!("Telegram website is reachable");
                },
                Ok(Err(e)) => {
                    eprintln!("Failed to reach Telegram website: {}", e);
                    play_alert_sound();
                    if let Err(e) = send_network_error(&client_clone).await {
                        eprintln!("Failed to send network error: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Timeout reaching Telegram website: {}", e);
                    play_alert_sound();
                    if let Err(e) = send_network_error(&client_clone).await {
                        eprintln!("Failed to send network error: {}", e);
                    }
                }
            }
            sleep(Duration::from_secs(60)).await;
        }
    });
    
    // Start Telegram badge monitoring task
    let client_clone2 = client.clone();
    tokio::spawn(async move {
        let mut last_count = String::new();
        loop {
            if let Some(count) = get_telegram_badge() {
                if count != last_count {
                    println!("🔔 检测到消息变化！当前未读数: {}", count);
                    play_alert_sound();
                    if let Err(e) = send_notification(&client_clone2).await {
                        eprintln!("Failed to send notification: {}", e);
                    }
                    last_count = count;
                }
            } else {
                // 如果返回 None，可能是 Telegram 没运行或没有未读消息
                if !last_count.is_empty() {
                    println!("✅ 消息已清空或程序已关闭。");
                    last_count = String::new();
                }
            }
            // 每 2 秒检查一次，避免过度占用 CPU
            sleep(Duration::from_secs(2)).await;
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

fn play_alert_sound() {
    // Use macOS afplay command to play a system sound
    if let Err(e) = Command::new("afplay").arg("/Users/dahuzi/Documents/pugongying.mp3").spawn() {
        eprintln!("Failed to play alert sound: {}", e);
    }
}