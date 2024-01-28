use std::{
    io::{Read, Write},
    net::TcpStream,
};

#[allow(dead_code)]
enum BitTorrentMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    Port,
}

pub struct PeerConnection {
    stream: TcpStream,
}

impl PeerConnection {
    pub fn new(peer_address: String, peer_port: u16) -> Self {
        let stream = TcpStream::connect(format!("{}:{}", peer_address, peer_port))
            .expect("Failed to connect to peer");
        PeerConnection { stream: stream }
    }

    pub fn handshake(&mut self, info_hash: &Vec<u8>, peer_id: &Vec<u8>) -> Vec<u8> {
        if info_hash.len() != 20 {
            panic!("Invalid info hash");
        }
        if peer_id.len() != 20 {
            panic!("Invalid peer id");
        }

        // Send handshake
        let mut handshake = Vec::new();
        handshake.push(19);
        handshake.extend(b"BitTorrent protocol");
        handshake.extend(vec![0; 8]);
        handshake.extend(info_hash);
        handshake.extend(peer_id);

        self.stream.write_all(&handshake).unwrap();

        // Receive handshake
        let mut handshake_response = [0 as u8; 68];
        self.stream.read(&mut handshake_response).unwrap();
        // parse handshake response

        let response_protocol_string_length = handshake_response[0];
        if response_protocol_string_length != 19 {
            panic!("Invalid protocol string length");
        }
        let protocol_string_offset = 1;
        let response_protocol_string: &[u8] = &handshake_response[protocol_string_offset
            ..response_protocol_string_length as usize + protocol_string_offset];
        if response_protocol_string != b"BitTorrent protocol" {
            panic!("Invalid protocol string");
        }

        let reserved_bytes_offset = response_protocol_string_length as usize + 1;
        let _response_reserved_bytes =
            &handshake_response[reserved_bytes_offset..reserved_bytes_offset + 8];

        let info_hash_offset = reserved_bytes_offset + 8;
        let response_info_hash = &handshake_response[info_hash_offset..info_hash_offset + 20];

        if response_info_hash != info_hash {
            panic!("Invalid info hash");
        }

        let peer_id_offset = info_hash_offset + 20;
        let response_peer_id = &handshake_response[peer_id_offset..peer_id_offset + 20];

        response_peer_id.to_vec()
    }

    // Add methods for peer communication
}
