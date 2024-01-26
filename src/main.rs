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
    } else if command == "info" {
        let filename = &args[2];
        // read file
        let content = std::fs::read(filename).expect("Something went wrong reading the file");
        let json_value: serde_json::Value = decode_bencode_to_json(&content);
        let tracker = json_value["announce"].to_string();
        let info = json_value["info"].to_string();

        println!("tracker: {}", tracker);
        println!("info: {}", info);
    } else {
        println!("unknown command: {}", args[1])
    }
}
