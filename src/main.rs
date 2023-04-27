use kickmyb_exploit::init_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_ui()?;

    // let resp = reqwest::get(SERVER_URL.to_owned() + "/index")
    //     .await?
    //     .json::<HashMap<String, String>>()
    //     .await?;
    // println!("{:#?}", resp);
    
    Ok(())
}