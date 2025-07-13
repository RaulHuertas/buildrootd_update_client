use error_chain::error_chain;
use std::collections::HashMap;
use std::io::Read;
use clap::Parser;
use std::fs;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use std::process::Command;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub server_base_url: String,
    #[arg(short, long)]
    pub network_interface : String,
    #[arg(short, long)]
    pub platform_info_file: String,
    #[arg(short, long)]
    pub download_path: String,
    #[arg(short, long, default_value_t = 2147483648)]
    pub max_download_size: i64,
    #[arg( long, default_value_t = 1_000_000)]
    pub download_buffer_size: i64,
}

fn get_mac_address(args: &Args) -> String {
    let mac_file_location = "/sys/class/net/".to_owned() + &args.network_interface + "/address";
    println!("Reading MAC address from: {}", mac_file_location);
    fs::read_to_string(mac_file_location).unwrap_or("00:00:00:00:00:00".to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MMPlatformInfo {
    pub version: i32,
    pub description: String,
    pub role: String,
}


#[derive(Deserialize,Debug,Serialize)]
pub struct CheckUpdateRequest{
    pub role : String,
    pub phy_id: String,
    pub description : String,
    pub installed_version : i32,
    pub last_updated_timestamp: chrono::DateTime<chrono::Utc>,
}

impl CheckUpdateRequest {
    pub fn new(
        role:&String,
        phy_id:&String
    )->Self{
       CheckUpdateRequest{
       role:role.clone(),
       phy_id:phy_id.clone(),
       description: "".to_string(),
       installed_version: 0,
       last_updated_timestamp : Local::now().into(),
       } 
    }

    pub fn load(
       platform_info:&MMPlatformInfo,
       args:&Args,
    )->Self{

       CheckUpdateRequest{
       role:platform_info.role.clone(),
       phy_id:get_mac_address(args),
       description: platform_info.description.clone(),
       installed_version: platform_info.version,
       last_updated_timestamp : Local::now().into(),
       } 
    }
}

#[derive(Deserialize, Default,Debug,Serialize)]
pub struct CheckUpdateResponse {
    pub update_available: bool
}

impl CheckUpdateResponse {
    pub fn new() -> Self {
        CheckUpdateResponse { update_available: false }
    }
} 

fn main() -> Result<()> {
    let args = Args::parse();
    
    let json_file_path = Path::new(args.platform_info_file.as_str());
    let file = File::open(json_file_path).expect("platform file not found");
    let platform_info :MMPlatformInfo = serde_json::from_reader(file).expect("error while reading");

    let dev = CheckUpdateRequest::load(&platform_info, &args);

    let client = reqwest::blocking::Client::new();
    let res = client.post(args.server_base_url.clone()+"/deviceCheckUpdate")
    .json(&dev)
    .header("Content-Type", "application/json")
    .send()?;

    let update_response = res.json::<CheckUpdateResponse>().unwrap();
    println!("Update available: {}", update_response.update_available);
    if !update_response.update_available {
        println!("No update available.");
        return Ok(());
    }
    //There is a new update available, download it
    let download_header_client = reqwest::blocking::Client::new(); 
    let dowload_header_response = download_header_client.head(args.server_base_url.clone() + "/downloadUpdate")
        .send()?;
    if !dowload_header_response.status().is_success() {
        println!("Failed to get download header, trying later: {}", dowload_header_response.status());
        return Ok(());
    }
    let headers = dowload_header_response.headers();
    let content_length : String = headers.get("content-length").unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let download_size = content_length.parse::<i64>().unwrap_or(0);
    if download_size <= 0 {
        println!("No content to download.");
        return Ok(());
    }
    if download_size > args.max_download_size {
        println!("Download size is too big");
        return Ok(());
    }
    
    //remove old file if exists
    let download_file = args.download_path.clone()+"/update.raucb";
    println!("Downloading update to: {}", download_file);
    println!("Download size: {} bytes", download_size);
    if Path::new(&download_file).exists() {
        fs::remove_file(&download_file)?;
    }
    let mut downloaded_range : i64 = 0;
    while downloaded_range < download_size {
        let to_download = download_size - downloaded_range;
        let to_download_now = std::cmp::min(to_download, args.download_buffer_size);

        let mut download_response = download_header_client.get(args.server_base_url.clone() + "/downloadUpdate")
            .header("Range", format!("bytes={}-{}", downloaded_range, downloaded_range + to_download_now - 1))
            .send()?;
        
        if !download_response.status().is_success() {
            println!("Failed to download update, trying later: {}", download_response.status());
            return Ok(());
        }
        
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&download_file)?;
        
        let mut buffer = Vec::new();
        download_response.read_to_end(&mut buffer)?;
        file.write_all(&buffer)?;
        file.flush()?;
        
        downloaded_range += to_download_now;
        println!("Downloaded {} bytes, total: {}", downloaded_range, download_size);

    }
    //////////////////////////////
    //Install the update
    //////////////////////////////
    let update_command_result = Command::new("rauc")
        .arg("install")
        .arg(download_file.clone())
        .status().expect("failed to execute rauc install");
     //println!("update commmand result: {} ",update_command_result);
     //update_command_result.expect("failed to execute rauc install");

    //////////////////////////////
    //Reboot the system to apply the update
    //////////////////////////////
    Command::new("reboot")
        .status()
        .expect("failed to reboot after install");

    Ok(())

}


