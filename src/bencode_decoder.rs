use serde_json;

enum BencodedValue {
    String(String),
    Integer(i64),
    List(Vec<BencodedValue>),
    Dictionary(Vec<(String, BencodedValue)>),
}

impl BencodedValue {
    fn to_json(&self) -> serde_json::Value {
        match self {
            BencodedValue::String(string) => serde_json::Value::String(string.to_string()),
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
            BencodedValue::String(string) => string.len().to_string().len() + 1 + string.len(), // <length>:<string>
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

fn decode_bencoded_string(encoded_value: &str) -> String {
    // Example: "5:hello" -> "hello"
    let colon_index = encoded_value.find(':').unwrap();
    let number_string = &encoded_value[..colon_index];
    let number = number_string.parse::<i64>().unwrap();
    let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    string.to_string()
}

fn decode_bencoded_integer(encoded_value: &str) -> i64 {
    // Example: "i3e" -> 3
    let end_index = encoded_value.find('e').unwrap();
    let number_string = &encoded_value[1..end_index];
    let number = number_string.parse::<i64>().unwrap();
    number
}

fn check_key_unique(key: &str, dictionary: &Vec<(String, BencodedValue)>) -> bool {
    for (k, _) in dictionary.iter() {
        if k == key {
            return false;
        }
    }
    true
}
fn decode_bencoded_value(encoded_value: &str) -> BencodedValue {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_digit(10) {
        return BencodedValue::String(decode_bencoded_string(encoded_value));
    } else if encoded_value.chars().next().unwrap() == 'i' {
        return BencodedValue::Integer(decode_bencoded_integer(encoded_value));
    } else if encoded_value.chars().next().unwrap() == 'l' {
        // Example: "l4:spam4:eggse" -> ["spam", "eggs"]
        let mut list = Vec::new();
        let mut index = 1;
        while encoded_value.chars().nth(index).unwrap() != 'e' && index < encoded_value.len() {
            let value = decode_bencoded_value(&encoded_value[index..]);
            let inc_size = value.get_byte_length();
            index += inc_size;
            list.push(value);
        }
        return BencodedValue::List(list);
    } else if encoded_value.chars().next().unwrap() == 'd' {
        // Example: "d3:cow3:moo4:spam4:eggse" -> {"cow": "moo", "spam": "eggs"}
        let mut dictionary = Vec::new();
        let mut index = 1;
        while encoded_value.chars().nth(index).unwrap() != 'e' && index < encoded_value.len() {
            //  Keys must be strings and appear once and only once.
            let key = decode_bencoded_string(&encoded_value[index..]);
            if check_key_unique(&key, &dictionary) == false {
                panic!("Key {} is not unique", key)
            }

            let inc_size = key.len().to_string().len() + 1 + key.len(); // <length>:<string>
            index += inc_size;

            let value = decode_bencoded_value(&encoded_value[index..]);
            let inc_size = value.get_byte_length();
            index += inc_size;
            dictionary.push((key, value));
        }
        return BencodedValue::Dictionary(dictionary);
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

pub fn decode_bencode_to_json(encoded_value: &str) -> serde_json::Value {
    let decoded_value = decode_bencoded_value(encoded_value);
    decoded_value.to_json()
}
