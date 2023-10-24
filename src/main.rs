use std::{env, fs, process};
use reqwest::blocking::Client;
use std::path::Path;

const BOOK_IO_API_ENDPOINT: &str = "https://api.book.io/api/v0/collections";
const BLOCKFROST_IPFS_GATEWAY: &str = "https://ipfs.blockfrost.io/api/v0/ipfs/gateway/";
const CARDANO_ASSETS_ENDPOINT: &str = "https://cardano-mainnet.blockfrost.io/api/v0/assets/";
const CARDANO_ASSETS_POLICY_ENDPOINT: &str = "https://cardano-mainnet.blockfrost.io/api/v0/assets/policy/";

#[derive(Debug, serde::Deserialize)]
struct BookIoCollection {
    collection_id: String,
    blockchain: String,
}

fn is_book_io_policy(policy_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Create a reqwest client
    let client = Client::new();

    // Specify the API endpoint URL
    let api_url = BOOK_IO_API_ENDPOINT;

    // Make a GET request to the API endpoint
    let json_response: serde_json::Value = client.get(api_url).send()?
        .error_for_status()?
        .json()?;

    // Deserialize the "data" array into Vec<BookIoCollection>
    let collections: Vec<BookIoCollection> = serde_json::from_value(json_response["data"].clone())?;

    // Check if the policy ID exists in the collections
    for collection in collections {
        if policy_id == collection.collection_id && collection.blockchain == "cardano" {
            return Ok(true);
        }
    }
    // The policy ID does not exist in the collections
    Ok(false)
}

fn ipfs_download(ipfs_path: &str, book_name: &str, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Accessing the environment variable directly inside the function
    let ipfs_project_id = match env::var("IPFS_PROJECT_ID") {
        Ok(key) => key,
        Err(_) => {
            return Err(From::from("Environment variable IPFS_PROJECT_ID is not set"));
        }
    };

    // Specify the URL for the IPFS file download
    let url = format!("{}{}", BLOCKFROST_IPFS_GATEWAY, ipfs_path);

    // Create a reqwest client
    let client = Client::new();

    // Make an HTTP GET request with headers to download the file
    let response = client
        .get(url)
        .header("Accept", "application/octet-stream")
        .header("project_id", ipfs_project_id)
        .send()?
        .error_for_status()?;

    // if the output directory does not exist, create it
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // get the data from the response
    let content = response.bytes()?;

    // construct the file path
    let file_path = output_dir.join(format!("{}.png", book_name));

    // If the file already exists, assume it's downloaded completely and return.
    // This is a simplification; more robust handling could include validating file integrity.
    if file_path.exists() {
        println!("File {:?} already exists, skipping download.", file_path);
        return Ok(());
    }

    // Write to file
    fs::write(&file_path, content)?;

    println!("PNG file saved to {:?}", file_path);
    Ok(())
}

fn get_highres_cover_ipfs_link (asset_id: &str, project_id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Specify the URL for cardano asset endpoint
    let url = format!("{}{}", CARDANO_ASSETS_ENDPOINT, asset_id);

    // Create a reqwest client
    let client = Client::new();

    // Make an HTTP GET request with headers to download the file
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header("project_id", project_id)
        .send()?
        .error_for_status()?;

    let json_response: serde_json::Value = response.json()?;

    // get the ipfs:// link for the high res cover image
    let src = json_response["onchain_metadata"]["files"][0]["src"].as_str().unwrap_or_default();
    // remove ipfs:// part from the string
    let ipfs_path = src.strip_prefix("ipfs://").expect("Invalid input format");
    // get the name of the book to be downloaded
    let book_name = json_response["onchain_metadata"]["name"].as_str().unwrap_or_default();

    Ok((ipfs_path.to_string(), book_name.to_string()))
}

fn choose_10_assets_of_a_policy (policy_id: &str, project_id: &str, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Specify the URL for the cardano asset policy endpoint
    let url = format!("{}{}", CARDANO_ASSETS_POLICY_ENDPOINT, policy_id);

    // Create a reqwest client
    let client = Client::new();

    // Make an HTTP GET request with headers to download the file
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header("project_id", project_id)
        .send()?
        .error_for_status()?;

    let json_response: serde_json::Value = response.json()?;
    // get the length of this array, if its less than 10, we quit the program
    let length = json_response.as_array().map_or(0, |arr| arr.len());

    if length > 11 {
        for i in 1..11 {
            let asset_id = json_response[i]["asset"].as_str().unwrap_or_default();
            // now retrieve the ipfs path and book name
            match get_highres_cover_ipfs_link(asset_id, project_id) {
                Ok((ipfs_path, book_name)) => {
                    // download the book covers
                    let _ = ipfs_download(&ipfs_path, &book_name, output_dir);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }
    else {
        eprintln!("ERROR: This policy_id has only {} assets", {length});
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if the correct number of arguments is provided
    if args.len() != 3 {
        eprintln!("ERROR: 2 arguments expected, {} given", args.len() - 1);
        process::exit(1); // Use std::process to exit the program in an error state
    }

    let policy_id = &args[1];
    let output_dir = Path::new(&args[2]);

    // Instead of hardcoded values, try to retrieve the environment variables for the project IDs.
    // Inform the user if the environment variables are not set.
    let cardano_project_id = match env::var("CARDANO_PROJECT_ID") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: Environment variable CARDANO_PROJECT_ID not set. Please set this variable with your project ID.");
            process::exit(1);
        }
    };

    // Check if the provided policy ID belongs to Book.io
    match is_book_io_policy(policy_id) {
        Ok(true) => {
            println!("{} is a valid Book.io policy id", policy_id);
            // select 10 assets associated with this policy id and do the rest inside this function
            let _ = choose_10_assets_of_a_policy(policy_id, &cardano_project_id, output_dir);
        }
        Ok(false) => {
            eprintln!("ERROR: Either {} is not a valid Book.io policy id or it is for evm", policy_id);
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Error: {}", err)
        }
    }
}
