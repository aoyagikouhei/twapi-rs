//! Account Activity API
extern crate base64;
extern crate crypto;

use self::crypto::mac::Mac;

pub fn calc_hmac(key: &str, input: &str) -> String {
    let mut hmac = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), key.as_bytes());
    hmac.input(input.as_bytes());
    base64::encode(hmac.result().code())
}

pub fn check_signature(signature: &str, consumer_secret: &str, body: &str) -> bool {
    let calced_body = calc_hmac(consumer_secret, body);
    signature == format!("sha256={}", calced_body)
}

pub fn make_crc_token_response(consumer_secret: &str, crc_token: &str) -> String {
    let calced = calc_hmac(consumer_secret, crc_token);
    format!("{{\"response_token\":\"sha256={}\"}}", calced)
}