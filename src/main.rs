use error_chain::error_chain;
use std::collections::HashMap;
use std::io::Read;
use clap::{Parser, ArgGroup};
use std::fs;
use chrono::{Local, DateTime};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    #[arg(short, long)]
    network_interface : String,
    #[arg(short, long)]
    description: String,
}

fn get_mac_address(args: &Args) -> String {
    fs::read_to_string("/sys/class/net/".to_owned() + &args.network_interface + "/address").unwrap_or("00:00:00:00:00:00".to_string())
}

#[derive(Deserialize,Debug,Serialize)]
struct CheckUpdateArguments{
    pub role : String,
    pub phy_id: String,
    pub description : String,
    pub installed_version : i32,
    pub last_updated_timestamp: chrono::DateTime<chrono::Utc>,
}

impl CheckUpdateArguments{
    pub fn new(
        role:&String,
        phy_id:&String
    )->Self{
       CheckUpdateArguments{
       role:role.clone(),
       phy_id:phy_id.clone(),
       description: "".to_string(),
       installed_version: 0,
       last_updated_timestamp : Local::now().into(),
       } 
    }

    pub fn load(
        role:&String,
        phy_id:&String
    )->Self{
       let description = fs::read_to_string("/opt/mmdescription").unwrap_or("No description".to_string());

       CheckUpdateArguments{
       role:role.clone(),
       phy_id:phy_id.clone(),
       description: description.to_string(),
       installed_version: 0,
       last_updated_timestamp : Local::now().into(),
       } 
    }
}

fn check_update_request(args:&Args)-> HashMap<String, String> {
    let mut body_data = HashMap::<String,String>::new();
    body_data.insert("role".to_string(), "kiosk".to_string());
    body_data.insert("phy_id".to_string(), get_mac_address(args));
    body_data.insert("description".to_string(), args.description.clone());
    body_data.insert("installed_version".to_string(), "0".to_string()); 
    body_data
}

fn main() -> Result<()> {
    let args = Args::parse();
    let device_info = check_update_request(&args);
    let mut mac_address = get_mac_address(&args);
    let dev = CheckUpdateArguments::new(&args.network_interface, &mac_address);

    let client = reqwest::blocking::Client::new();
    let mut res = client.post(args.server_base_url+"/deviceCheckUpdate")
    .json(&dev)
    .header("Content-Type", "application/json")
    .send()?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    Ok(())
}


