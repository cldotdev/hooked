use anyhow::Result;
use hex::decode_to_slice;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

/// Returns the `X-Hub-Signature-256` header value for `message`.
pub fn sign(secret: &[u8], message: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)?;
    mac.update(message);
    Ok(format!(
        "sha256={}",
        hex::encode(mac.finalize().into_bytes())
    ))
}

pub fn verify_signature(secret: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)?;
    mac.update(message);

    let mut dec_signature = [0u8; 32];
    decode_to_slice(signature, &mut dec_signature)?;

    if mac.verify_slice(&dec_signature).is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_signature_returns_true() {
        // Testing data is from
        // https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#testing-the-webhook-payload-validation
        let secret = b"It's a Secret to Everybody";
        let payload = b"Hello, World!";
        let exp_sig = b"757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";
        let result = verify_signature(secret, payload, exp_sig).unwrap();

        assert!(result);
    }

    #[test]
    fn sign_matches_github_test_vector() {
        let secret = b"It's a Secret to Everybody";
        let payload = b"Hello, World!";
        assert_eq!(
            sign(secret, payload).unwrap(),
            "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17"
        );
    }

    #[test]
    fn verify_signature_returns_false() {
        let secret = b"It's a Secret to Everybody";
        let payload = b"Hello, World!";
        let exp_sig = b"03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4";
        let result = verify_signature(secret, payload, exp_sig).unwrap();

        assert!(!result);
    }

    #[test]
    fn verify_signature_returns_invalid_length_error() {
        let secret = b"It's a Secret to Everybody";
        let payload = b"Hello, World!";
        let exp_sig = b"1234";
        let result = verify_signature(secret, payload, exp_sig).unwrap_err();

        assert_eq!(format!("{}", result), "Invalid string length");
    }
}
