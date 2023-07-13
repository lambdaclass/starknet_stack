use cairo_felt::Felt252;
use serde::{Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

pub struct FeltHex;

pub struct FeltHexOption;

pub struct FeltPendingBlockHash;

pub(crate) struct NumAsHex;

impl SerializeAs<Felt252> for FeltHex {
    fn serialize_as<S>(value: &Felt252, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = value.to_str_radix(16);
        serializer.serialize_str(&value)
    }
}

impl<'de> DeserializeAs<'de, Felt252> for FeltHex {
    fn deserialize_as<D>(deserializer: D) -> Result<Felt252, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Felt252::parse_bytes(value.as_bytes(), 16).unwrap())
    }
}

impl SerializeAs<Option<Felt252>> for FeltHexOption {
    fn serialize_as<S>(value: &Option<Felt252>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_str(&value.to_str_radix(16)),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> DeserializeAs<'de, Option<Felt252>> for FeltHexOption {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<Felt252>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "" => Ok(None),
            _ => Ok(Some(Felt252::from_bytes_be(value.as_bytes()))),
        }
    }
}

impl SerializeAs<Option<Felt252>> for FeltPendingBlockHash {
    fn serialize_as<S>(value: &Option<Felt252>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_str(&value.to_str_radix(16)),
            // We don't know if it's `null` or `"pending"`
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> DeserializeAs<'de, Option<Felt252>> for FeltPendingBlockHash {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<Felt252>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.is_empty() || value == "pending" || value == "None" {
            Ok(None)
        } else {
            Ok(Some(Felt252::from_bytes_be(value.as_bytes())))
        }
    }
}

impl SerializeAs<u64> for NumAsHex {
    fn serialize_as<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{value:#x}"))
    }
}

impl<'de> DeserializeAs<'de, u64> for NumAsHex {
    fn deserialize_as<D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match u64::from_str_radix(&value[2..], 16) {
            Ok(value) => Ok(value),
            Err(err) => Err(serde::de::Error::custom(format!(
                "invalid hex string: {err}"
            ))),
        }
    }
}
pub mod base64 {
    use base64::engine::general_purpose;
    use base64::Engine;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = general_purpose::STANDARD.encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        general_purpose::STANDARD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::{InvokeTransaction, InvokeTransactionV1, Transaction};

    use super::*;

    use serde_with::serde_as;

    #[serde_as]
    #[derive(Deserialize)]
    struct TestStruct(#[serde_as(as = "FeltHexOption")] pub Option<Felt252>);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn empty_string_deser() {
        let r = serde_json::from_str::<TestStruct>("\"\"").unwrap();
        assert_eq!(r.0, None);
    }

    #[test]
    fn serialize_deserialize_tx() {
        let starknet_transaction =
            Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {
                transaction_hash: Felt252::new(2314),
                max_fee: Felt252::new(1),
                signature: vec![Felt252::new(423)],
                nonce: Felt252::new(1),
                sender_address: Felt252::new(1),
                calldata: vec![Felt252::new(2)],
            }));

        let starknet_transaction_str = serde_json::to_string(&starknet_transaction).unwrap();

        let deserialized_tx: Transaction = serde_json::from_str(&starknet_transaction_str).unwrap();
        if let Transaction::Invoke(InvokeTransaction::V1(v)) = deserialized_tx {
            assert_eq!(Felt252::new(2314), v.transaction_hash);
            assert_eq!(&Felt252::new(423), v.signature.first().unwrap());
        } else {
            panic!()
        }
    }
}
