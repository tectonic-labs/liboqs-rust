
#[cfg(feature = "falcon")]
#[test]
fn keypair_from_seed() {
    use oqs::sig::*;
    const SEED: &[u8] = &[1u8; 48];

    let sig = Sig::new(Algorithm::Falcon512).unwrap();
    assert!(sig.keypair_from_seed(SEED).is_ok());
}