use crate::crypt::{TEST_KEY, decrypt_aes256gcm, encrypt_aes256gcm};
use crate::packet::Packet;

mod crypt;
pub mod error;
mod packet;

pub fn encrypt(data: &[u8]) -> error::Result<Vec<u8>> {
    //todo remove TEST_KEY
    let (ciphertext, nonce) = encrypt_aes256gcm(TEST_KEY, data)?;
    let packet = Packet {
        nonce,
        data: ciphertext,
    };
    packet.encode()
}

pub fn decrypt(data: &[u8]) -> error::Result<Vec<u8>> {
    let packet = Packet::decode(data)?;
    //todo remove TEST_KEY
    let decrypted = decrypt_aes256gcm(TEST_KEY, &packet.data, &packet.nonce)?;
    Ok(decrypted)
}

// pub fn encrypt(data: &[u8]) -> error::Result<Vec<u8>> {
//     let (ciphertext, nonce) = encrypt_aes256gcm(TEST_KEY, data)?;
//     let mut p = Vec::with_capacity(12 + ciphertext.len());
//     p.extend_from_slice(&nonce);
//     p.extend_from_slice(&ciphertext);
//     Ok(p)
// }
// 
// pub fn decrypt(data: &[u8]) -> error::Result<Vec<u8>> {
//     let nonce = &data[..12];// [u8; 12]
//     let ciphertext = &data[12..];
//     let decrypted = decrypt_aes256gcm(TEST_KEY, ciphertext, &nonce)?;
//     Ok(decrypted)
// }

#[test]
fn proto_test() {
    let text = b"hello world";
    let send_packet = encrypt(text).unwrap();
    let receive_packet = decrypt(send_packet.as_slice()).unwrap();
    assert_eq!(receive_packet, b"hello world");
    
    let result = String::from_utf8(receive_packet).unwrap();
    println!("{:?}", result);
}
