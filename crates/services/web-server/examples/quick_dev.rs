#![allow(unused)]

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; 

use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:8080")?;

	// hc.do_get("/index.html").await?.print().await?;

	// -- Register 
	let req_login = hc.do_post(
		"/api/register",
		json!({
			"username": "testusername",
			"pwd": "testpwd"
		}),
	);
	req_login.await?.print().await?;

	// -- Login
	let req_login = hc.do_post(
		"/api/login",
		json!({
			"username": "testusername",
			"pwd": "testpwd"
		}),
	);
	req_login.await?.print().await?;

	Ok(())
}
