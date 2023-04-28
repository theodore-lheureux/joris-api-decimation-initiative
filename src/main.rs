use std::{collections::HashMap, env};

use kickmyb_exploit::{init_ui, SERVER_URL};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SigninRes {
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = &mut env::args();
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    let username = args.skip(1).next().unwrap_or_else(|| {
        eprintln!("Usage: <username> <password>");
        std::process::exit(1);
    });
    let password = args.next().unwrap_or_else(|| {
        eprintln!("Usage: <username> <password>");
        std::process::exit(1);
    });

    let mut map = HashMap::new();
    map.insert("username", username);
    map.insert("password", password);

    let mut resp = client
        .post(SERVER_URL.to_owned() + "id/signin")
        .json(&map)
        .send()
        .await?;
    
    if resp.status().as_u16() != 200 {
        resp = client
        .post(SERVER_URL.to_owned() + "id/signup")
        .json(&map)
        .send()
        .await?;

        if resp.status().as_u16() != 200 {
            eprintln!("Error: {}", resp.text().await?);
            std::process::exit(1);
        }
    }

    init_ui(client, resp.json::<SigninRes>().await?.username).await?;

    Ok(())
}
