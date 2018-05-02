//! This is the documentation for the `LSW` scheme:
//!
//! * Developped by Allison Lewko, Amit Sahai and Brent Waters, "Revocation Systems with Very Small Private Keys"
//! * Published in Security and Privacy, 2010. SP'10. IEEE Symposium on. IEEE
//! * Available from http://eprint.iacr.org/2008/309.pdf
//! * Type: encryption (key-policy attribute-based)
//! * Setting: bilinear groups (asymmetric)
//! * Authors: Georg Bramm
//! * Date:	04/2018
//!
//! # Examples
//!
//! ```
//!use rabe::schemes::lsw::*;
//!let (pk, msk) = setup();
//!let plaintext = String::from("our plaintext!").into_bytes();
//!let policy = String::from(r#"{"OR": [{"ATT": "X"}, {"ATT": "B"}]}"#);
//!let ct_kp: KpAbeCiphertext = encrypt(&pk, &vec!["A".to_string(), "B".to_string()], &plaintext).unwrap();
//!let sk: KpAbeSecretKey = keygen(&pk, &msk, &policy).unwrap();
//!assert_eq!(decrypt(&sk, &ct_kp).unwrap(), plaintext);
//! ```
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate bn;
extern crate rand;
extern crate byteorder;
extern crate crypto;
extern crate bincode;
extern crate num_bigint;
extern crate blake2_rfc;

use std::string::String;
use bn::*;
use std::ops::Neg;
use utils::tools::*;
use utils::secretsharing::{gen_shares_str, calc_coefficients_str, calc_pruned_str};
use utils::aes::*;
use utils::hash::{blake2b_hash_fr, blake2b_hash_g1};

/// A LSW Public Key (PK)
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct KpAbePublicKey {
    _g_g1: bn::G1,
    _g_g2: bn::G2,
    _g_g1_b: bn::G1,
    _g_g1_b2: bn::G1,
    _h_g1_b: bn::G1,
    _e_gg_alpha: bn::Gt,
}

/// A LSW Master Key (MSK)
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct KpAbeMasterKey {
    _alpha1: bn::Fr,
    _alpha2: bn::Fr,
    _b: bn::Fr,
    _h_g1: bn::G1,
    _h_g2: bn::G2,
}

/// A LSW Secret User Key (SK)
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct KpAbeSecretKey {
    _policy: String,
    _dj: Vec<(String, bn::G1, bn::G2, bn::G1, bn::G1, bn::G1)>,
}

/// A LSW Ciphertext (CT)
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct KpAbeCiphertext {
    _e1: bn::Gt,
    _e2: bn::G2,
    _ej: Vec<(String, bn::G1, bn::G1, bn::G1)>,
    _ct: Vec<u8>,
}

/// The setup algorithm of LSW KP-ABE. Generates a new KpAbePublicKey and a new KpAbeMasterKey.
pub fn setup() -> (KpAbePublicKey, KpAbeMasterKey) {
    // random number generator
    let _rng = &mut rand::thread_rng();
    // generate random alpha1, alpha2 and b
    let _alpha1 = Fr::random(_rng);
    let _alpha2 = Fr::random(_rng);
    let _beta = Fr::random(_rng);
    let _alpha = _alpha1 * _alpha2;
    let _g_g1 = G1::random(_rng);
    let _g_g2 = G2::random(_rng);
    let _h_g1 = G1::random(_rng);
    let _h_g2 = G2::random(_rng);
    let _g1_b = _g_g1 * _beta;
    // calculate the pairing between g1 and g2^alpha
    let _e_gg_alpha = pairing(_g_g1, _g_g2).pow(_alpha);

    // set values of PK
    let _pk = KpAbePublicKey {
        _g_g1: _g_g1,
        _g_g2: _g_g2,
        _g_g1_b: _g1_b,
        _g_g1_b2: _g1_b * _beta,
        _h_g1_b: _h_g1 * _beta,
        _e_gg_alpha: _e_gg_alpha,
    };
    // set values of MSK
    let _msk = KpAbeMasterKey {
        _alpha1: _alpha1,
        _alpha2: _alpha2,
        _b: _beta,
        _h_g1: _h_g1,
        _h_g2: _h_g2,
    };
    // return PK and MSK
    return (_pk, _msk);
}

/// The key generation algorithm of LSW KP-ABE.
/// Generates a KpAbeSecretKey using a KpAbePublicKey, a KpAbeMasterKey and a policy given as JSON String.
///
/// # Arguments
///
///	* `_pk` - A Public Key (PK), generated by the function setup()
///	* `_msk` - A Master Key (MSK), generated by the function setup()
///	* `_policy` - An access policy given as JSON String
///
pub fn keygen(
    _pk: &KpAbePublicKey,
    _msk: &KpAbeMasterKey,
    _policy: &String,
) -> Option<KpAbeSecretKey> {
    // random number generator
    let _rng = &mut rand::thread_rng();
    let _shares = gen_shares_str(_msk._alpha1, _policy).unwrap();
    let mut _d: Vec<(String, bn::G1, bn::G2, bn::G1, bn::G1, bn::G1)> = Vec::new();
    for (_share_str, _share_value) in _shares.into_iter() {
        let _r = Fr::random(_rng);
        if is_negative(&_share_str) {
            _d.push((
                _share_str.to_string(),
                G1::zero(),
                G2::zero(),
                (_pk._g_g1 * _share_value) + (_pk._g_g1_b2 * _r),
                _pk._g_g1_b * (blake2b_hash_fr(&_share_str) * _r) +
                    (_msk._h_g1 * _r),
                _pk._g_g1 * _r.neg(),
            ));
        } else {
            _d.push((
                _share_str.to_string(),
                (_pk._g_g1 * (_msk._alpha2 * _share_value)) +
                    (blake2b_hash_g1(_pk._g_g1, &_share_str) * _r),
                _pk._g_g2 * _r,
                G1::zero(),
                G1::zero(),
                G1::zero(),
            ));
        }
    }
    return Some(KpAbeSecretKey {
        _policy: _policy.clone(),
        _dj: _d,
    });
}

/// The encrypt algorithm of LSW KP-ABE. Generates a new KpAbeCiphertext using an KpAbePublicKey, a set of attributes given as String Vector and some plaintext data given as [u8].
///
/// # Arguments
///
///	* `_pk` - A Public Key (PK), generated by the function setup()
///	* `_attributes` - A set of attributes given as String Vector
///	* `_plaintext` - plaintext data given as a Vector of u8
///
pub fn encrypt(
    _pk: &KpAbePublicKey,
    _attributes: &Vec<String>,
    _plaintext: &[u8],
) -> Option<KpAbeCiphertext> {
    if _attributes.is_empty() || _plaintext.is_empty() {
        return None;
    } else {
        // random number generator
        let _rng = &mut rand::thread_rng();
        // attribute vector
        let mut _ej: Vec<(String, bn::G1, bn::G1, bn::G1)> = Vec::new();
        // random secret
        let _s = Fr::random(_rng);
        // sx vector
        let mut _sx: Vec<(bn::Fr)> = Vec::new();
        _sx.push(_s);
        for (_i, _attr) in _attributes.iter().enumerate() {
            _sx.push(Fr::random(_rng));
            _sx[0] = _sx[0] - _sx[_i];
        }
        for (_i, _attr) in _attributes.into_iter().enumerate() {
            _ej.push((
                _attr.to_string(),
                blake2b_hash_g1(_pk._g_g1, &_attr) * _s,
                _pk._g_g1_b * _sx[_i],
                (_pk._g_g1_b2 * (_sx[_i] * blake2b_hash_fr(&_attr))) +
                    (_pk._h_g1_b * _sx[_i]),
            ));
        }
        // random message
        let _msg = pairing(G1::random(_rng), G2::random(_rng));
        //Encrypt plaintext using derived key from secret
        return Some(KpAbeCiphertext {
            _e1: _pk._e_gg_alpha.pow(_s) * _msg,
            _e2: _pk._g_g2 * _s,
            _ej: _ej,
            _ct: encrypt_symmetric(&_msg, &_plaintext.to_vec()).unwrap(),
        });

    }
}

/// The decrypt algorithm of LSW KP-ABE. Reconstructs the original plaintext data as Vec<u8>, given a KpAbeCiphertext with a matching KpAbeSecretKey.
///
/// # Arguments
///
///	* `_sk` - A Secret Key (SK), generated by the function keygen()
///	* `_ct` - A LSW KP-ABE Ciphertext
///
pub fn decrypt(_sk: &KpAbeSecretKey, _ct: &KpAbeCiphertext) -> Option<Vec<u8>> {
    let _attrs_str = _ct._ej
        .iter()
        .map(|values| values.clone().0.to_string())
        .collect::<Vec<_>>();
    let _pruned = calc_pruned_str(&_attrs_str, &_sk._policy);
    match _pruned {
        None => {
            return None;
        }
        Some(_p) => {
            let (_match, _list) = _p;
            if _match {
                let mut _prod_t = Gt::one();
                let mut _z_y = Gt::one();
                let _coeffs: Vec<(String, bn::Fr)> = calc_coefficients_str(&_sk._policy).unwrap();
                for _attr_str in _list.iter() {
                    let _sk_attr = _sk._dj
                        .iter()
                        .filter(|_attr| _attr.0 == _attr_str.to_string())
                        .nth(0)
                        .unwrap();
                    let _ct_attr = _ct._ej
                        .iter()
                        .filter(|_attr| _attr.0 == _attr_str.to_string())
                        .nth(0)
                        .unwrap();
                    let _coeff_attr = _coeffs
                        .iter()
                        .filter(|_attr| _attr.0 == _attr_str.to_string())
                        .nth(0)
                        .unwrap();
                    if is_negative(&_attr_str) {
                        // TODO !!
		                /*let _sum_e4 = G2::zero();
		                let _sum_e5 = G2::zero();
		                _prod_t = _prod_t *
		                    (pairing(sk._d_i[_i].3, ct._e2) *
		                         (pairing(sk._d_i[_i].4, _sum_e4) * pairing(sk._d_i[_i].5, _sum_e5))
		                             .inverse());
		                */
                    } else {
                        _z_y = pairing(_sk_attr.1, _ct._e2) *
                            pairing(_ct_attr.1, _sk_attr.2).inverse();
                    }
                    _prod_t = _prod_t * _z_y.pow(_coeff_attr.1);
                }
                let _msg = _ct._e1 * _prod_t.inverse();
                // Decrypt plaintext using derived secret from cp-abe scheme
                return decrypt_symmetric(&_msg, &_ct._ct);
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn and() {
        // setup scheme
        let (pk, msk) = setup();
        // a set of two attributes matching the policy
        let mut att_matching: Vec<String> = Vec::new();
        att_matching.push(String::from("A"));
        att_matching.push(String::from("B"));
        att_matching.push(String::from("C"));
        // our plaintext
        let plaintext = String::from("dance like no one's watching, encrypt like everyone is!")
            .into_bytes();
        // our policy
        let policy = String::from(r#"{"AND": [{"ATT": "C"}, {"ATT": "B"}]}"#);
        // kp-abe ciphertext
        let ct_kp_matching: KpAbeCiphertext = encrypt(&pk, &att_matching, &plaintext).unwrap();
        // a kp-abe SK key
        let sk: KpAbeSecretKey = keygen(&pk, &msk, &policy).unwrap();
        // and now decrypt again with matching sk
        assert_eq!(decrypt(&sk, &ct_kp_matching).unwrap(), plaintext);
    }

    #[test]
    fn or() {
        // setup scheme
        let (pk, msk) = setup();
        // a set of two attributes matching the policy
        let mut att_matching: Vec<String> = Vec::new();
        att_matching.push(String::from("A"));
        att_matching.push(String::from("B"));
        att_matching.push(String::from("C"));
        // our plaintext
        let plaintext = String::from("dance like no one's watching, encrypt like everyone is!")
            .into_bytes();
        // our policy
        let policy = String::from(r#"{"OR": [{"ATT": "X"}, {"ATT": "B"}]}"#);
        // kp-abe ciphertext
        let ct_kp_matching: KpAbeCiphertext = encrypt(&pk, &att_matching, &plaintext).unwrap();
        // a kp-abe SK key
        let sk: KpAbeSecretKey = keygen(&pk, &msk, &policy).unwrap();
        // and now decrypt again with matching sk
        assert_eq!(decrypt(&sk, &ct_kp_matching).unwrap(), plaintext);
    }

    #[test]
    fn or_and() {
        // setup scheme
        let (pk, msk) = setup();
        // a set of two attributes matching the policy
        let mut att_matching: Vec<String> = Vec::new();
        att_matching.push(String::from("A"));
        att_matching.push(String::from("Y"));
        att_matching.push(String::from("Z"));
        // our plaintext
        let plaintext = String::from("dance like no one's watching, encrypt like everyone is!")
            .into_bytes();
        // our policy
        let policy = String::from(
            r#"{"OR": [{"ATT": "X"}, {"AND": [{"ATT": "Y"}, {"ATT": "Z"}]}]}"#,
        );
        // kp-abe ciphertext
        let ct_kp_matching: KpAbeCiphertext = encrypt(&pk, &att_matching, &plaintext).unwrap();
        // a kp-abe SK key
        let sk: KpAbeSecretKey = keygen(&pk, &msk, &policy).unwrap();
        // and now decrypt again with matching sk
        assert_eq!(decrypt(&sk, &ct_kp_matching).unwrap(), plaintext);
    }

    #[test]
    fn not() {
        // setup scheme
        let (pk, msk) = setup();
        // a set of two attributes matching the policy
        let mut att_matching: Vec<String> = Vec::new();
        att_matching.push(String::from("A"));
        att_matching.push(String::from("B"));
        // our plaintext
        let plaintext = String::from("dance like no one's watching, encrypt like everyone is!")
            .into_bytes();
        // our policy
        let policy = String::from(r#"{"OR": [{"ATT": "X"}, {"ATT": "Y"}]}"#);
        // kp-abe ciphertext
        let ct_kp_matching: KpAbeCiphertext = encrypt(&pk, &att_matching, &plaintext).unwrap();
        // a kp-abe SK key
        let sk: KpAbeSecretKey = keygen(&pk, &msk, &policy).unwrap();
        // and now decrypt again with matching sk
        assert_eq!(decrypt(&sk, &ct_kp_matching), None);
    }
}
