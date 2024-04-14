use lib_utils::time::now_utc_plus_sec_str;
use tokio::{fs::{self, File}, io::AsyncWriteExt};

use super::error::CoreError;

pub async fn upload_file(path: &str, data: &[u8]) -> Result<String, CoreError> {
    let img_name = now_utc_plus_sec_str(1.0);

    let mut file = File::create(format!("{}/{}.png", path, img_name)).await?;

    file.write(&data).await?;
    let file_url = format!("uploads/{}.png", img_name);

    Ok(file_url)
}

pub async fn remove_file(url: String) -> Result<(), CoreError> {
    fs::remove_file(url).await?;

    Ok(())
}