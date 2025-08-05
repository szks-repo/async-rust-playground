use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

pub async fn read_file(filename: &str) -> Result<String, std::io::Error> {
    let mut file = tokio::fs::File::open(filename).await?;
    let mut src = String::new();
    file.read_to_string(&mut src).await?;

    Ok(src)
}

pub fn get_desktop_file_path(file_name: &str) -> Result<PathBuf, std::io::Error> {
    if let Some(desktop_path) = dirs::desktop_dir() {
        let full_path = desktop_path.join(file_name);
        Ok(full_path)
    } else {
        // PathBuf が取得できなかった場合 (None の場合)
        // `NotFound` 種類のエラーを新しく作成します
        Err(Error::new(ErrorKind::NotFound, "Could not find the desktop directory."))
    }
}