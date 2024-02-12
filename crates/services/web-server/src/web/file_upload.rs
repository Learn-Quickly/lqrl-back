use crate::web::Result;
use axum::body::Bytes;

use tokio::{fs::File, io::AsyncWriteExt, };

use chrono::Utc;

pub async fn upload_file(data: Bytes) -> Result<String> {
    let img_name: i64 = Utc::now().timestamp(); 
    let mut file = File::create(format!("./public/uploads/{}.png",img_name)).await.unwrap();
    file.write(&data).await.unwrap();
    let file_url = format!("uploads/{}.png", img_name);

    Ok(file_url)
}