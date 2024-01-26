use serde_json;
use std::env;

mod bencode_decoder;
use bencode_decoder::decode_bencode_to_json;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let json_value: serde_json::Value = decode_bencode_to_json(encoded_value);
        println!("{}", json_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
