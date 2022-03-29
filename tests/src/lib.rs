use sp_core::{hexdisplay::HexDisplay};
use sp_runtime::{AccountId32};
use sp_std::str::{FromStr, Utf8Error};


pub fn get_account_encoded(sub_address: &str) -> Option<String> {
    let account_id = AccountId32::from_str(sub_address);
    match account_id {
        Ok(id) => {
            let address = format!("{:?}", HexDisplay::from(&id.as_ref()));
            Some(address)
        },
        Err(_) => None,
    }
}

#[test]
fn test() {
    let acc = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let encode = get_account_encoded(acc).unwrap();
    println!("encode: {:?}", encode);
}

#[test]
fn get_account_encoded_works() {
    let ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let encode = get_account_encoded(ALICE).unwrap();
    assert_eq!(encode, "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
}

#[test]
fn get_account_encoded_fail() {
    let BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let encode = get_account_encoded(BOB).unwrap();
    assert_ne!(encode, "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
}

#[test]
fn get_account_encoded_error() {
    let NONAME = "asasdas";
    let encode = get_account_encoded(NONAME);
    assert_eq!(encode, None);
}