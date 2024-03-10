use crate::web::Result;
use axum::body::Bytes;

use tokio::{fs::{self, File}, io::AsyncWriteExt, };

use chrono::Utc;

use super::Error;

pub async fn upload_file(data: Bytes) -> Result<String> {
    let img_name: i64 = Utc::now().timestamp(); 
    let mut file = File::create(format!("./public/uploads/{}.png",img_name))
        .await
        .map_err(|_| Error::CreateFileFail)?;

    file.write(&data).await.unwrap();
    let file_url = format!("uploads/{}.png", img_name);

    Ok(file_url)
}

pub async fn remove_file(url: String) -> Result<()> {
    fs::remove_file(url)
        .await
        .map_err(|_| Error::RemoveFileFail)?;

    Ok(())
}