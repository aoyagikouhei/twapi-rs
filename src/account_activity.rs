//! Account Activity API
use self::crypto::mac::Mac;
use self::ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;

pub fn calc_hmac(key: &str, input: &str) -> String {
    let mut hmac = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), key.as_bytes());
    hmac.input(input.as_bytes());
    base64::encode(hmac.result().code())
}

pub fn check_signature(signature: &str, consumer_secret: &str, body: &str) -> bool {
    let calced_body = calc_hmac(consumer_secret, body);
    signature == format!("sha256={}", calced_body)
}

pub fn check_ip(ip: &str) -> bool {
    let net1: Ipv4Network = "199.59.148.0/22".parse().unwrap();
    let net2: Ipv4Network = "199.16.156.0/22".parse().unwrap();
    let target: Ipv4Addr = ip.parse().unwrap();
    net1.contains(target) || net2.contains(target)
}

pub fn make_crc_token_response(consumer_secret: &str, crc_token: &str) -> String {
    let calced = calc_hmac(consumer_secret, crc_token);
    format!("{{\"response_token\":\"sha256={}\"}}", calced)
}
