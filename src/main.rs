mod config;
mod notifier;
mod bybit;

use anyhow::Result;
use chrono::{DateTime, Local, Timelike};
use std::time::Duration;
use tokio::time;
use tracing::{info, error, Level};
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::Config;
use crate::notifier::pushover::PushoverNotifier;
use crate::bybit::api::BybitAPI;

fn get_next_notification_time(current_time: DateTime<Local>, notification_times: &[u32]) -> u32 {
    let current_seconds = current_time.hour() * 3600 + current_time.minute() * 60 + current_time.second();
    
    for &time in notification_times {
        if time > current_seconds {
            return time;
        }
    }
    
    notification_times[0]
}

// 秒数を時刻形式に変換する関数
fn seconds_to_time_string(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_ansi(true)
        .pretty()
        .init();
    
    dotenv::dotenv()?;
    
    let config = Config::load()?;
    
    let notifier = PushoverNotifier::new(
        std::env::var("PUSHOVER_TOKEN")?,
        std::env::var("PUSHOVER_USER_KEY")?,
    );
    
    let bybit_client = BybitAPI::new();

    info!("FR通知システムを開始します...");
    info!("監視対象通貨ペア: {:?}", config.symbols);
    
    let notification_times_str: Vec<String> = config.notification_times
        .iter()
        .map(|&t| seconds_to_time_string(t))
        .collect();
    info!("設定された通知時刻: {:?}", notification_times_str);
    info!("デバッグプッシュモード: {}", if config.debug_push { "有効" } else { "無効" });

    // デバッグモードが有効な場合、起動時に1回テスト通知を実行
    if config.debug_push {
        info!("デバッグモードが有効です。テスト通知を実行します...");
        for symbol in &config.symbols {
            info!("{}のFRを取得中...", symbol);
            match bybit_client.get_funding_rate(symbol).await {
                Ok(fr) => {
                    let message = format!("[デバッグ通知] {} の現在のFR: {}%", symbol, fr);
                    info!("{}のFR: {}%", symbol, fr);
                    match notifier.send(&message).await {
                        Ok(_) => info!("{}のデバッグ通知を送信しました", symbol),
                        Err(e) => error!("{}のデバッグ通知送信に失敗: {}", symbol, e),
                    }
                }
                Err(e) => {
                    error!("{}のFR取得に失敗: {}", symbol, e);
                }
            }
        }
        info!("デバッグ通知の送信が完了しました");
    }

    let now = Local::now();
    let next_time = get_next_notification_time(now, &config.notification_times);
    info!("次回の通知時刻: {}", seconds_to_time_string(next_time));

    loop {
        let now = Local::now();
        let seconds_since_midnight = now.hour() * 3600 + now.minute() * 60 + now.second();

        // 設定された時間になったらFRを取得して通知
        if config.notification_times.contains(&seconds_since_midnight) {
            info!("通知時刻になりました。FRの取得を開始します...");
            for symbol in &config.symbols {
                info!("{}のFRを取得中...", symbol);
                match bybit_client.get_funding_rate(symbol).await {
                    Ok(fr) => {
                        let message = format!("{} の現在のFR: {}%", symbol, fr);
                        info!("{}のFR: {}%", symbol, fr);
                        match notifier.send(&message).await {
                            Ok(_) => info!("{}のFR通知を送信しました", symbol),
                            Err(e) => error!("{}の通知送信に失敗: {}", symbol, e),
                        }
                    }
                    Err(e) => {
                        error!("{}のFR取得に失敗: {}", symbol, e);
                    }
                }
            }
            
            let now = Local::now();
            let next_time = get_next_notification_time(now, &config.notification_times);
            info!("次回の通知時刻: {}", seconds_to_time_string(next_time));
        }

        time::sleep(Duration::from_secs(1)).await;
    }
}