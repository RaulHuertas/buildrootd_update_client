
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    dotenvy::dotenv().expect("Unable to access .env file");
    // build our application with a single route
    let server_url = std::env::var("SERVER_ADDR").unwrap_or("localhost:3335".to_owned());


    let mut res = reqwest::blocking::get(server_url+"/deviceCheckUpdates")?;

    eprintln!("Response: {:?} {}", res.version(), res.status());
    eprintln!("Headers: {:#?}\n", res.headers());

    // copy the response body directly to stdout
    res.copy_to(&mut std::io::stdout())?;

    Ok(())
}

