use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rodio::{Decoder, OutputStream, Sink};
use serde::Deserialize;
use tungstenite::{connect, Error as WsError, Message};
use url::Url;

use reqwest::Client;
use tokio::runtime::Runtime;

// 配置结构
#[derive(Deserialize, Clone)]
struct PairConfig {
    high: f64,
    low: f64,
}

#[derive(Deserialize, Clone)]
struct Config {
    pairs: HashMap<String, PairConfig>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置文件的路径
    let config_path = Path::new("config.toml");

    // 使用Arc和Mutex共享配置
    let config = Arc::new(Mutex::new(load_config(config_path)?));

    // 克隆Arc以用于监视线程
    let config_clone = Arc::clone(&config);

    // 设置文件监视器以实现热更新
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if event.kind.is_modify() {
                        println!("Config file modified, reloading...");
                        if let Ok(new_config) = load_config(config_path) {
                            let mut config = config_clone.lock().unwrap();
                            *config = new_config;
                            println!("Config reloaded successfully.");
                        } else {
                            println!("Failed to reload config.");
                        }
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(config_path, RecursiveMode::NonRecursive)?;

    // 克隆Arc以用于WebSocket线程
    let config_clone_ws = Arc::clone(&config);

    // 启动WebSocket连接线程
    thread::spawn(move || {
        if let Err(e) = monitor_prices(config_clone_ws) {
            println!("Error in WebSocket thread: {:?}", e);
        }
    });

    // 主线程无限循环，等待Ctrl+C退出
    println!("Program running. Press Ctrl+C to exit...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

// 加载配置
fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

// 监控价格（添加重连和超时检测）
fn monitor_prices(config: Arc<Mutex<Config>>) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_duration = Duration::from_secs(60); // 1分钟超时

    loop {
        // 构建订阅的streams（每次重连时重新构建，以支持热更新后的新pairs）
        let mut streams = Vec::new();
        {
            let config = config.lock().unwrap();
            for pair in config.pairs.keys() {
                streams.push(format!("{}@ticker", pair.to_lowercase()));
            }
        }

        let ws_url = format!(
            "wss://stream.binance.com:9443/ws/{}",
            streams.join("/")
        );

        // Parse the URL to validate it, but pass the string to connect
        Url::parse(&ws_url)?;

        let connect_result = connect(&ws_url);

        match connect_result {
            Ok((mut socket, _)) => {
                println!("Connected to Binance WebSocket.");

                let mut last_price_time = Instant::now(); // 初始化在这里，避免警告

                loop {
                    // 检查超时
                    if Instant::now().duration_since(last_price_time) > timeout_duration {
                        println!("No price update for over 1 minute, alerting and reconnecting...");
                        play_alert_sound()?; // 超时播放警报
                        break; // 跳出内循环，重连
                    }

                    match socket.read() {
                        Ok(msg) => {
                            if let Message::Text(text) = msg {
                                // 解析Ticker数据
                                if let Ok(ticker) = serde_json::from_str::<Ticker>(&text) {
                                    let price: f64 = ticker.c.parse()?;
                                    println!("Current price for {}: {}", ticker.s, price); // 实时打印价格

                                    last_price_time = Instant::now(); // 更新最后价格时间

                                    let config = config.lock().unwrap();
                                    if let Some(pair_config) = config.pairs.get(&ticker.s) {
                                        if price > pair_config.high || price < pair_config.low {
                                            println!(
                                                "Alert: {} price {} is out of range [{}, {}]",
                                                ticker.s, price, pair_config.low, pair_config.high
                                            );
                                            play_alert_sound()?;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("WebSocket error: {:?}", e);
                            if let WsError::ConnectionClosed = e {
                                println!("WebSocket disconnected, alerting and reconnecting...");
                                play_alert_sound()?; // 断开时播放警报
                            }
                            break; // 跳出内循环，重连
                        }
                    }

                    // 短暂睡眠，避免高CPU
                    thread::sleep(Duration::from_millis(100));
                }

                // 重连前等待一段时间
                thread::sleep(Duration::from_secs(5));
            }
            Err(e) => {
                println!("Failed to connect: {:?}", e);
                play_alert_sound()?; // 连接失败时播放警报
                thread::sleep(Duration::from_secs(10)); // 等待10秒重试
            }
        }
    }
}

// Binance Ticker结构（简化）
#[derive(Deserialize)]
struct Ticker {
    s: String, // 符号，如BTCUSDT
    c: String, // 当前价格
}

// 发送预警到Telegram
async fn send_alert() -> anyhow::Result<()> {
    let client = Client::new();
    let url = "https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=BTC%20price%20alert%20triggered!";
    client.get(url).send().await?;
    Ok(())
}

// 播放警报声音（假设有一个alert.wav文件）
fn play_alert_sound() -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let file = File::open("/Users/dahuzi/Documents/pugongying.mp3")?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    
    // 发送预警（添加错误处理，确保即使发送失败也不会影响程序运行）
    if let Ok(rt) = Runtime::new() {
        if let Err(e) = rt.block_on(send_alert()) {
            println!("Failed to send alert: {:?}", e);
        }
    } else {
        println!("Failed to create runtime for sending alert");
    }
    
    sink.sleep_until_end();
    Ok(())
}