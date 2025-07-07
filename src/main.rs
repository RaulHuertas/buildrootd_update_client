use error_chain::error_chain;
use std::collections::HashMap;
use std::io::Read;
use clap::{Parser, ArgGroup};
use std::env;
use std::fs;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    server_base_url: String,
    network_interface : String,
    description: String,
}

fn get_mac_address(args: &Args) -> String {
    fs::read_to_string("/sys/class/net/".to_owned() + &args.network_interface + "/address").unwrap_or("00:00:00:00:00:00".to_string())
}

fn check_update_request(args:&Args)-> HashMap<String, String> {
    //create a hashmap to hold the body data
    //let mut body_data = HashMap::new();
    let mut body_data = HashMap::<String,String>::new();
    body_data.insert("key1".to_string(), "value1".to_string());
    body_data.insert("key2".to_string(), "value2".to_string());

    body_data.insert("role".to_string(), "kiosk".to_string());
    body_data.insert("phy_id".to_string(), get_mac_address(args));
    body_data.insert("role".to_string(), args.description.clone());


    body_data
}

fn main() -> Result<()> {
    let args = Args::parse();

    let client = reqwest::blocking::Client::new();
    let mut res = client.post(args.server_base_url+"/deviceCheckUpdate")
    .body("test")
    .header("Content-Type", "application/json")
    .send()?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    Ok(())
}


