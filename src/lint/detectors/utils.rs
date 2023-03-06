use move_core_types::account_address::AccountAddress;

pub fn fmt_address_hex(value: &str) -> String {
    AccountAddress::from_hex_literal(value).unwrap().to_canonical_string()
}