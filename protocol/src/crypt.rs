use crate::error;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};

/// Data encryption with AES-256-GCM
pub fn encrypt_aes256gcm(
    key: &[u8],       // 32 bytes (256 bits)
    plaintext: &[u8], // encryption data
) -> error::Result<(Vec<u8>, Vec<u8>)> {
    // Создаем шифр с заданным ключом
    let cipher = Aes256Gcm::new_from_slice(key)?;

    // Генерируем уникальный nonce (96 бит/12 байт)
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Шифруем данные
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    Ok((ciphertext, nonce.to_vec()))
}

/// Data decryption with AES-256-GCM
pub fn decrypt_aes256gcm(
    key: &[u8],        // 32 bytes (256 bits)
    ciphertext: &[u8], // encrypted data
    nonce: &[u8],      // 12 bytes
) -> error::Result<Vec<u8>> {
    // расшифрованные данные
    // Создаем шифр с заданным ключом
    let cipher = Aes256Gcm::new_from_slice(key)?;

    // Создаем nonce из переданного значения
    let nonce = Nonce::from_slice(nonce);

    // Дешифруем данные
    let plaintext = cipher.decrypt(nonce, ciphertext)?;

    Ok(plaintext)
}

#[test]
fn crypt_test() -> error::Result<()> {
    const TEST_KEY: &[u8] = b"32_byte_secret_key_for_aes256gcm";
    
    assert_eq!(TEST_KEY.len(), 32, "Key must be exactly 32 bytes");

    // Данные для шифрования
    let plaintext = b"Confidential VPN data packet";
    println!("Original: {:?}", String::from_utf8_lossy(plaintext));

    // Шифруем
    let (ciphertext, nonce) = encrypt_aes256gcm(TEST_KEY, plaintext)?;
    println!("Encrypted: {:?}", ciphertext);

    // Дешифруем
    let decrypted = decrypt_aes256gcm(TEST_KEY, &ciphertext, &nonce)?;
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));

    // Проверяем, что данные совпадают
    assert_eq!(plaintext, decrypted.as_slice());

    Ok(())
}
