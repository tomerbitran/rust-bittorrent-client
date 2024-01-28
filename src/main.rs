use bstr::BString;
use hex::encode;
use rand::Rng;
use serde_json;
use sha1::{Digest, Sha1};
use std::env;

mod bencode_decoder;
use bencode_decoder::BencodedValue;
use bencode_decoder::*;

mod peer_comm;
use peer_comm::PeerConnection;

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

fn url_encode_hash(hash: &str) -> String {
    let bytes: Vec<u8> = (0..hash.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hash[i..i + 2], 16).unwrap())
        .collect();
    bytes.iter().map(|b| format!("%{:02X}", b)).collect()
}

fn get_random_peer_id() -> String {
    let mut peer_id = String::from("-TR2920-");
    let mut rng = rand::thread_rng();
    for _ in 0..12 {
        let random_number = rng.gen_range(0..10);
        peer_id.push_str(&random_number.to_string());
    }
    peer_id
}

fn get_peers(
    announce_url: &str,
    info_hash: &str,
    peer_id: &str,
    port: u16,
    length: u64,
) -> Result<Vec<String>, reqwest::Error> {
    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded=0&downloaded=0&left={}&compact=1",
        announce_url,
        url_encode_hash(info_hash),
        peer_id,
        port,
        length
    );

    let response = reqwest::blocking::get(&url)?;
    let body = response.bytes().expect("error reading response");
    let decoded_body = decode_bencoded_value(&body.to_vec());
    let dictionary = decoded_body.extract_dict().unwrap();
    let peers = dictionary
        .get(&BString::from("peers"))
        .expect("error in response")
        .extract_bstring()
        .expect("invalid peers??");

    let mut index = 0;
    let mut peer_list = Vec::new();
    while index < peers.len() - 5 {
        let ip = format!(
            "{}.{}.{}.{}",
            peers[index],
            peers[index + 1],
            peers[index + 2],
            peers[index + 3]
        );
        let port = u16::from_be_bytes([peers[index + 4], peers[index + 5]]);
        let peer = format!("{}:{}", ip, port);
        peer_list.push(peer);
        index += 6;
    }
    Ok(peer_list)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <command> <filename>", args[0]);
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let encoded_bytes = encoded_value.as_bytes();
            let json_value: serde_json::Value = decode_bencode_to_json(&encoded_bytes.to_vec());
            println!("{}", json_value.to_string());
        }
        "info" => {
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
        }
        "peers" => {
            let filename = &args[2];
            let content = std::fs::read(filename).expect("Something went wrong reading the file");
            let parsed_file = decode_bencoded_value(&content);
            let torrent_file = TorrentFile::new(&parsed_file);

            let client_peer_id = get_random_peer_id(); // length of this string must be 20
            let port = 6881;
            let info_hash = torrent_file.info.info_hash;
            let announce_url = torrent_file.announce;
            let file_length = torrent_file.info.length;
            let tracker_response = get_peers(
                &announce_url,
                &info_hash,
                client_peer_id.as_str(),
                port,
                file_length,
            );
            match tracker_response {
                Ok(peers) => {
                    println!("Peers:");
                    for peer in peers.iter() {
                        println!("  {}", peer);
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        "handshake" => {
            let filename = &args[2];
            let peer_address_port = &args[3]; // <address>:<port>
            let peer_address_port: Vec<&str> = peer_address_port.split(':').collect();
            let peer_address = peer_address_port[0];
            let peer_port = peer_address_port[1].parse::<u16>().expect("invalid port");

            let content = std::fs::read(filename).expect("Something went wrong reading the file");
            let parsed_file = decode_bencoded_value(&content);
            let torrent_file = TorrentFile::new(&parsed_file);

            let client_peer_id = get_random_peer_id(); // length of this string must be 20
            let info_hash_bytes = (0..torrent_file.info.info_hash.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&torrent_file.info.info_hash[i..i + 2], 16).unwrap())
                .collect();
            let mut peer_connection = PeerConnection::new(peer_address.to_string(), peer_port);

            let remote_peer_id =
                peer_connection.handshake(&info_hash_bytes, &client_peer_id.as_bytes().to_vec());
            println!("Peer ID:{}", encode(&remote_peer_id));
        }
        _ => println!("unknown command: {}", args[1]),
    }
}
