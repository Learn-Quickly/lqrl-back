use axum::body::Bytes;

use tokio::{fs::{self, File}, io::AsyncWriteExt, };

use chrono::Utc;

use crate::error::{AppError, AppResult};

pub async fn upload_file(data: Bytes) -> AppResult<String> {
    let img_name: i64 = Utc::now().timestamp(); 
    let mut file = File::create(format!("./public/uploads/{}.png",img_name))
        .await
        .map_err(|_| AppError::CreateFileFail)?;

    file.write(&data).await.unwrap();
    let file_url = format!("uploads/{}.png", img_name);

    Ok(file_url)
}

pub async fn remove_file(url: String) -> AppResult<()> {
    fs::remove_file(url)
        .await
        .map_err(|_| AppError::RemoveFileFail)?;

    Ok(())
}