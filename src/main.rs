use hex::encode;
use serde_json;
use sha1::{Digest, Sha1};
use std::env;

mod bencode_decoder;
use bencode_decoder::{decode_bencode_to_json, encode_json_to_bencode};

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let encoded_bytes = encoded_value.as_bytes();
        let json_value: serde_json::Value = decode_bencode_to_json(&encoded_bytes.to_vec());
        println!("{}", json_value.to_string());
    } else if command == "info" {
        let filename = &args[2];
        // read file
        let content = std::fs::read(filename).expect("Something went wrong reading the file");
        let json_value: serde_json::Value = decode_bencode_to_json(&content);
        let tracker = json_value["announce"].to_string();
        let length = json_value["info"]["length"].to_string();

        let info_dict = &json_value["info"];
        let json_bytes = encode_json_to_bencode(&info_dict);

        let mut hasher = Sha1::new();
        hasher.update(json_bytes);
        let hash_result = hasher.finalize();

        let info_hash = encode(hash_result);
        println!("Tracker URL: {}", tracker);
        println!("Length: {}", length);
        println!("Info hash: {}", info_hash);
    } else {
        println!("unknown command: {}", args[1])
    }
}
