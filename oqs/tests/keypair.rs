use oqs::sig::*;
use rstest::*;

#[rstest]
#[cfg_attr(feature = "falcon", case::falcon512(Algorithm::Falcon512, &[1u8; 48]))]
#[cfg_attr(feature = "falcon", case::falcon1024(Algorithm::Falcon1024, &[1u8; 48]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa44(Algorithm::MlDsa44, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa65(Algorithm::MlDsa65, &[1u8; 32]))]
#[cfg_attr(feature = "ml_dsa", case::mldsa87(Algorithm::MlDsa87, &[1u8; 32]))]
fn keypair_from_seed(#[case] algorithm: Algorithm, #[case] seed: &[u8]) {
    let sig = Sig::new(algorithm).unwrap();
    let res = sig.keypair_from_seed(seed);
    assert!(res.is_ok(), "{:?}", res.unwrap_err());
}
