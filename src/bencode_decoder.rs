use bstr::{BString, ByteSlice};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BencodedValue {
    BString(BString),
    Integer(i64),
    List(Vec<BencodedValue>),
    Dictionary(HashMap<BString, BencodedValue>),
}

impl BencodedValue {
    // Function to forcibly extract

    pub fn extract_dict(&self) -> Option<&HashMap<BString, BencodedValue>> {
        match self {
            BencodedValue::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn extract_bstring(&self) -> Option<&BString> {
        match self {
            BencodedValue::BString(bstring) => Some(bstring),
            _ => None,
        }
    }

    pub fn extract_integer(&self) -> Option<i64> {
        match self {
            BencodedValue::Integer(val) => Some(*val),
            _ => None,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoded_value = Vec::new();
        match self {
            BencodedValue::BString(bstring) => {
                encoded_value.extend(bstring.len().to_string().as_bytes());
                encoded_value.push(':' as u8);
                encoded_value.extend(bstring.as_bytes());
            }
            BencodedValue::Integer(number) => {
                encoded_value.push('i' as u8);
                encoded_value.extend(number.to_string().as_bytes());
                encoded_value.push('e' as u8);
            }
            BencodedValue::List(list) => {
                encoded_value.push('l' as u8);
                for value in list.iter() {
                    encoded_value.extend(value.encode());
                }
                encoded_value.push('e' as u8);
            }
            BencodedValue::Dictionary(map) => {
                encoded_value.push('d' as u8);
                // sort by key
                let mut map: Vec<_> = map.iter().collect();
                map.sort_by(|a, b| a.0.cmp(b.0));

                for (key, value) in map.iter() {
                    encoded_value.extend(key.len().to_string().as_bytes());
                    encoded_value.push(':' as u8);
                    encoded_value.extend(key.as_bytes());

                    encoded_value.extend(value.encode());
                }
                encoded_value.push('e' as u8);
            }
        }
        encoded_value
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            BencodedValue::BString(bstring) => serde_json::Value::Array(
                bstring
                    .to_vec()
                    .iter()
                    .map(|byte| serde_json::Value::Number(serde_json::Number::from(*byte))).collect(),
            ),
            BencodedValue::Integer(number) => serde_json::Value::Number(serde_json::Number::from(
                number.to_string().parse::<i64>().unwrap(),
            )),
            BencodedValue::List(list) => {
                let mut json_list = Vec::new();
                for bencoded_value in list.iter() {
                    json_list.push(bencoded_value.to_json());
                }
                serde_json::Value::Array(json_list)
            }
            BencodedValue::Dictionary(dictionary) => {
                let mut json_dictionary = serde_json::Map::new();
                for (key, bencoded_value) in dictionary.iter() {
                    json_dictionary.insert(key.to_string(), bencoded_value.to_json());
                }
                serde_json::Value::Object(json_dictionary)
            }
        }
    }

    fn get_byte_length(&self) -> usize {
        match self {
            BencodedValue::BString(string) => string.len().to_string().len() + 1 + string.len(), // <length>:<string>
            BencodedValue::Integer(number) => number.to_string().len() + 2, // i<number>e
            BencodedValue::List(list) => {
                2 + list
                    .iter()
                    .map(|bencoded_value| bencoded_value.get_byte_length())
                    .sum::<usize>()
            } // l<list...>e
            BencodedValue::Dictionary(dictionary) => {
                2 + dictionary
                    .iter()
                    .map(|(key, bencoded_value)| {
                        key.len()
                            + 1
                            + key.len().to_string().len()
                            + bencoded_value.get_byte_length()
                    })
                    .sum::<usize>()
            } // d<dictionary...>e
        }
    }
}

fn decode_bencoded_bstring(encoded_value: &[u8]) -> bstr::BString {
    // Example: "5:hello" -> "hello"
    let colon_index = encoded_value.find_char(':').unwrap();
    let number_string = String::from_utf8_lossy(&encoded_value[..colon_index]);
    let number = number_string.parse::<i64>().expect("Invalid number ");
    let bstring = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    bstr::BString::from(bstring)
}

fn decode_bencoded_integer(encoded_value: &[u8]) -> i64 {
    // Example: "i3e" -> 3
    let end_index = encoded_value.find_char(char::from('e')).unwrap();
    let number_string = String::from_utf8_lossy(&encoded_value[1..end_index]);
    let number = number_string.parse::<i64>().unwrap();
    number
}

pub fn decode_bencoded_value(encoded_value: &[u8]) -> BencodedValue {
    // If encoded_value starts with a digit, it's a number
    let first_char = encoded_value.iter().next().unwrap();
    if first_char.is_ascii_digit() {
        return BencodedValue::BString(decode_bencoded_bstring(&encoded_value));
    } else if *first_char == 'i' as u8 {
        return BencodedValue::Integer(decode_bencoded_integer(encoded_value));
    } else if *first_char == 'l' as u8 {
        // Example: "l4:spam4:eggse" -> ["spam", "eggs"]
        let mut list = Vec::new();
        let mut index = 1;
        while *encoded_value.iter().nth(index).unwrap() != 'e' as u8 && index < encoded_value.len()
        {
            let value = decode_bencoded_value(&encoded_value[index..]);
            let inc_size = value.get_byte_length();
            index += inc_size;
            list.push(value);
        }
        return BencodedValue::List(list);
    } else if *first_char == 'd' as u8 {
        // Example: "d3:cow3:moo4:spam4:eggse" -> {"cow": "moo", "spam": "eggs"}
        let mut dictionary = HashMap::new();
        let mut index = 1;
        while *encoded_value.iter().nth(index).unwrap() != 'e' as u8 && index < encoded_value.len()
        {
            //  Keys must be strings and appear once and only once.
            let key = decode_bencoded_bstring(&encoded_value[index..]);
            if dictionary.contains_key(&key) {
                panic!("Key {} is not unique", key)
            }

            let inc_size: usize = key.len().to_string().len() + 1 + key.len(); // <length>:<string>
            index += inc_size;

            let value = decode_bencoded_value(&encoded_value[index..]);
            let inc_size = value.get_byte_length();
            index += inc_size;
            dictionary.insert(key, value);
        }
        return BencodedValue::Dictionary(dictionary);
    } else {
        panic!(
            "Unhandled encoded value: {}",
            String::from_utf8_lossy(encoded_value)
        )
    }
}

pub fn decode_bencode_to_json(encoded_value: &[u8]) -> serde_json::Value {
    let decoded_value = decode_bencoded_value(encoded_value);
    decoded_value.to_json()
}
