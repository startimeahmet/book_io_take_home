# Take Home Project Instructions

First, create a Blockfrost Cardano Mainnet project and take a note of the project ID.

For downloading the files from IPFS, I could not find any reliable APIs, therefore I used Blockfrost IPFS API to download the book covers. For that, you need to create a seperate project and take a note of that project ID as well.

I ran into issues with Blockfrost IPFS projects though as they kept suspending the accounts I opened. I don't know why that happened. I didn't want to pay and get their premium API access neither. So, I just kept opening new accounts once they suspended the existing one.

Before running the Rust program, set the environment variables with your project IDs. This way, there is no hardcoded sensitive information inside the source code.

For Linux:
```
export CARDANO_PROJECT_ID=your_cardano_project_id_here
export IPFS_PROJECT_ID=your_ipfs_project_id_here
```

For Windows:
```
setx CARDANO_PROJECT_ID "your_cardano_project_id_here"
setx IPFS_PROJECT_ID "your_ipfs_project_id_here"
```

You might need to restart your terminal or IDE for changes to take effect.

## Dependencies
You will need `reqwest`, `serde_json` and `serde` crates for this program to run. See the `Cargo.toml` which is already configured for the program to run.

## Running the program
```
cargo run -- <policy_id> <output_path>
```

### Example
`cargo run -- 477cec772adb1466b301fb8161f505aa66ed1ee8d69d3e7984256a43 covers`


## Notes
- The program is idempotent, i.e. it will skip the files that are already downloaded. If halted during a download, it will pick up where it left off. Though, if a specific file download is interrupted and the file is not fully downloaded, it will not attempt to detect this and redownload it. I did not implement it mainly because of time constraints. It can be done by checksum verification (using `sha256` in `onchain_metadata`).
- Error handling in the code can be improved significantly and the current version might have some issues i.e. the errors might not propagate all the way to the terminal to let the user know. However, I did not want to spend so much time perfecting these conditions again mainly because of the time constraints.
- Finally, I created a new instance of `reqwest::blocking::Client` in every function because we are not doing that many requests. We can consider creating a single client instance and passing it to the functions that need it.