use std::{collections::HashMap, env};

use joris_api_decimation_initiative::{init_ui, SERVER_URL};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SigninRes {
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = &mut env::args();
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    let arg_error = || {
        eprintln!("Usage: joris-api-decimation-initiative <username> <password>");
        std::process::exit(1);
    };

    let username = args.nth(1).unwrap_or_else(arg_error);
    let password = args.next().unwrap_or_else(arg_error);

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
