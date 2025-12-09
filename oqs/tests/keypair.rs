use oqs::kem::Kem;
use oqs::sig::*;
use rstest::*;

#[rstest]
#[cfg_attr(feature = "falcon", case::falcon512(Algorithm::Falcon512, &[1u8; 48]))]
#[cfg_attr(feature = "falcon", case::falcon1024(Algorithm::Falcon1024, &[1u8; 48]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa44(Algorithm::MlDsa44, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa65(Algorithm::MlDsa65, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa87(Algorithm::MlDsa87, &[1u8; 32]))]
fn sig_keypair_from_seed(#[case] algorithm: Algorithm, #[case] seed: &[u8]) {
    let sig = Sig::new(algorithm).unwrap();
    let res = sig.keypair_from_seed(seed);
    assert!(res.is_ok(), "{:?}", res.unwrap_err());
}

#[rstest]
#[cfg_attr(feature = "ml_kem", case::mlkem512(oqs::kem::Algorithm::MlKem512, &[1u8; 64]))]
#[cfg_attr(feature = "ml_kem", case::mlkem768(oqs::kem::Algorithm::MlKem768, &[1u8; 64]))]
#[cfg_attr(feature = "ml_kem", case::mlkem1024(oqs::kem::Algorithm::MlKem1024, &[1u8; 64]))]
#[cfg_attr(feature = "classic_mceliece", case::mceliece348864(oqs::kem::Algorithm::ClassicMcEliece348864, &[1u8; 32]))]
#[cfg_attr(feature = "classic_mceliece", case::mceliece348864f(oqs::kem::Algorithm::ClassicMcEliece348864f, &[1u8; 32]))]
fn kem_keypair_from_seed(#[case] algorithm: oqs::kem::Algorithm, #[case] seed: &[u8]) {
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
