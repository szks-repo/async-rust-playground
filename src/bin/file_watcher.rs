use std::path::{PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;
use tokio::time::sleep;
use async_rust_playground::{get_desktop_file_path, read_file};

type MainResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> MainResult<()> {
    let path_buf = get_desktop_file_path("test.txt")?;
    let (tx, mut rx) = watch::channel(false);

    tokio::spawn(watch_file_changes(tx, path_buf.clone()));
    loop {
        /// .changed() は `tx`がドロップされるとErrを返すため、
        /// Okの場合のみ処理を続けるようにする
        if rx.changed().await.is_ok() {
            if let Ok(contents) = read_file(path_buf.to_str().unwrap()).await {
                println!("--- File Updated ---\n{}", contents);
            }
        } else {
            // 送信側(tx)がドロップされたらループを抜ける
            break;
        }
    }

    Ok(())
}

async fn watch_file_changes(tx: watch::Sender<bool>, path: PathBuf) {
    println!("watch file changes start: {:?}", path);
    let mut last_modified: Option<SystemTime> = None;

    loop {
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if let Ok(modified) = metadata.modified() {
                if last_modified == None {
                    last_modified = Some(modified);
                    continue
                }
                if last_modified != Some(modified) {
                    println!("Change detected!");
                    last_modified = Some(modified);
                    if tx.send(true).is_err() {
                        println!("Receiver dropped. Watcher stopping.");
                        break;
                    }
                }
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
}