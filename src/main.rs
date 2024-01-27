use bstr::BString;
use hex::encode;
use serde_json;
use sha1::{Digest, Sha1};
use std::env;

mod bencode_decoder;
use bencode_decoder::BencodedValue;
use bencode_decoder::*;



#[allow(dead_code)]
struct TorrentFileInfo {
    info_hash: String,
    name: String,
    length: u64,
    piece_length: u64,
    raw_pieces: BString,
    piece_hashes: Vec<String>,
}

impl TorrentFileInfo {
    fn get_piece_hashes(pieces: &BString) -> Vec<String> {
        let mut piece_hashes = Vec::new();
        let mut index = 0;
        while index < pieces.len() {
            let piece_hash = encode(&pieces[index..index + 20]);

            piece_hashes.push(piece_hash);
            index += 20;
        }
        piece_hashes
    }

    fn new(bencoded_info: BencodedValue) -> TorrentFileInfo {
        let info_bytes = bencoded_info.encode();
        let dictionary = bencoded_info.extract_dict().unwrap();

        let name = String::from_utf8_lossy(
            dictionary[&BString::from("name")]
                .extract_bstring()
                .unwrap(),
        )
        .to_string();
        let length = dictionary[&BString::from("length")]
            .extract_integer()
            .unwrap();
        let piece_length = dictionary[&BString::from("piece length")]
            .extract_integer()
            .unwrap();
        let pieces = dictionary[&BString::from("pieces")]
            .extract_bstring()
            .unwrap()
            .clone();

        let mut hasher = Sha1::new();
        hasher.update(&info_bytes);
        let hash_result = hasher.finalize();

        let piece_hashes = TorrentFileInfo::get_piece_hashes(&pieces);
        TorrentFileInfo {
            info_hash: encode(hash_result),
            name: name,
            length: length as u64,
            piece_length: piece_length as u64,
            raw_pieces: pieces,
            piece_hashes: piece_hashes,
        }
    }
}
struct TorrentFile {
    announce: String,
    info: TorrentFileInfo,
}

impl TorrentFile {
    fn new(parsed_file: &BencodedValue) -> TorrentFile {
        let dictionary = parsed_file.extract_dict().unwrap();

        let announce = String::from_utf8_lossy(
            dictionary[&BString::from("announce")]
                .extract_bstring()
                .unwrap(),
        )
        .to_string();
        let info = TorrentFileInfo::new(dictionary[&BString::from("info")].clone());

        TorrentFile { announce, info }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <command> <filename>", args[0]);
        return;
    }

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
        let parsed_file = decode_bencoded_value(&content);
        let torrent_file = TorrentFile::new(&parsed_file);

        println!("Tracker URL: {}", torrent_file.announce);
        println!("Length: {}", torrent_file.info.length);
        println!("Info hash: {}", torrent_file.info.info_hash);
        println!("Piece Hashes:");
        for piece_hash in torrent_file.info.piece_hashes.iter() {
            println!("  {}", piece_hash);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
