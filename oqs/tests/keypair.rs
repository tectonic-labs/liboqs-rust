use rstest::*;
use tectonic_oqs::kem::Kem;
use tectonic_oqs::sig::*;

#[rstest]
#[cfg_attr(feature = "falcon", case::falcon512(Algorithm::Falcon512, &[1u8; 48]))]
#[cfg_attr(feature = "falcon", case::falcon1024(Algorithm::Falcon1024, &[1u8; 48]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa44(Algorithm::MlDsa44, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa65(Algorithm::MlDsa65, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa87(Algorithm::MlDsa87, &[1u8; 32]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_128s(Algorithm::SlhDsaPureSha2128s, &[1u8; 48]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_128f(Algorithm::SlhDsaPureSha2128f, &[1u8; 48]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_192s(Algorithm::SlhDsaPureSha2192s, &[1u8; 72]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_192f(Algorithm::SlhDsaPureSha2192f, &[1u8; 72]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_256s(Algorithm::SlhDsaPureSha2256s, &[1u8; 96]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_sha2_256f(Algorithm::SlhDsaPureSha2256f, &[1u8; 96]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_128s(Algorithm::SlhDsaPureShake128s, &[1u8; 48]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_128f(Algorithm::SlhDsaPureShake128f, &[1u8; 48]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_192s(Algorithm::SlhDsaPureShake192s, &[1u8; 72]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_192f(Algorithm::SlhDsaPureShake192f, &[1u8; 72]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_256s(Algorithm::SlhDsaPureShake256s, &[1u8; 96]))]
#[cfg_attr(feature = "slh_dsa", case::slhdsa_pure_shake_256f(Algorithm::SlhDsaPureShake256f, &[1u8; 96]))]
fn sig_keypair_from_seed(#[case] algorithm: Algorithm, #[case] seed: &[u8]) {
    let sig = Sig::new(algorithm).unwrap();
    let res = sig.keypair_from_seed(seed);
    assert!(res.is_ok(), "{:?}", res.unwrap_err());
}

#[rstest]
#[cfg_attr(feature = "ml_kem", case::mlkem512(tectonic_oqs::kem::Algorithm::MlKem512, &[1u8; 64]))]
#[cfg_attr(feature = "ml_kem", case::mlkem768(tectonic_oqs::kem::Algorithm::MlKem768, &[1u8; 64]))]
#[cfg_attr(feature = "ml_kem", case::mlkem1024(tectonic_oqs::kem::Algorithm::MlKem1024, &[1u8; 64]))]
#[cfg_attr(feature = "classic_mceliece", case::mceliece348864(tectonic_oqs::kem::Algorithm::ClassicMcEliece348864, &[1u8; 32]))]
#[cfg_attr(feature = "classic_mceliece", case::mceliece348864f(tectonic_oqs::kem::Algorithm::ClassicMcEliece348864f, &[1u8; 32]))]
fn kem_keypair_from_seed(#[case] algorithm: tectonic_oqs::kem::Algorithm, #[case] seed: &[u8]) {
    let kem = Kem::new(algorithm).unwrap();
    let s = kem.keypair_seed_from_bytes(&seed).unwrap();
    let (pk1, sk1) = kem.keypair_derand(s).unwrap();
    let (pk2, sk2) = kem.keypair_derand(s).unwrap();

    assert_eq!(pk1, pk2);
    assert_eq!(sk1, sk2);
    let (pk3, sk3) = kem.keypair().unwrap();
    assert_ne!(pk1, pk3);
    assert_ne!(sk1, sk3);
}
