//! Transaction types
use super::{rlp_opt, NUM_TX_FIELDS};
use crate::{
    types::{Address, Bloom, Bytes, Log, H256, U256, U64},
    utils::keccak256,
};
use rlp::RlpStream;
use serde::{Deserialize, Serialize};

/// Details of a signed transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U64>,

    /// Transaction Index. None when pending.
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<U64>,

    /// Sender
    pub from: Address,

    /// Recipient (None when contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,

    /// Transfered value
    pub value: U256,

    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,

    /// Gas amount
    pub gas: U256,

    /// Input data
    pub input: Bytes,

    /// ECDSA recovery id
    pub v: U64,

    /// ECDSA signature r
    pub r: U256,

    /// ECDSA signature s
    pub s: U256,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "feeCurrency")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "gatewayFeeRecipient"
    )]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "gatewayFee")]
    pub gateway_fee: Option<U256>,
}

impl Transaction {
    // modifies the RLP stream with the Celo-specific information
    // This is duplicated from TransactionRequest. Is there a good way to get rid
    // of this code duplication?
    #[cfg(feature = "celo")]
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, self.fee_currency);
        rlp_opt(rlp, self.gateway_fee_recipient);
        rlp_opt(rlp, self.gateway_fee);
    }

    pub fn hash(&self) -> H256 {
        keccak256(&self.rlp().as_ref()).into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        rlp.append(&self.nonce);
        rlp.append(&self.gas_price);
        rlp.append(&self.gas);

        #[cfg(feature = "celo")]
        self.inject_celo_metadata(&mut rlp);

        rlp_opt(&mut rlp, self.to);
        rlp.append(&self.value);
        rlp.append(&self.input.as_ref());
        rlp.append(&self.v);
        rlp.append(&self.r);
        rlp.append(&self.s);

        rlp.out().freeze().into()
    }
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: H256,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U64,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U64>,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<U256>,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<U64>,
    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub root: Option<H256>,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Bloom,
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
mod tests {
    use super::*;

    #[test]
    fn decode_transaction_response() {
        let _res: Transaction = serde_json::from_str(
            r#"{
    "blockHash":"0x1d59ff54b1eb26b013ce3cb5fc9dab3705b415a67127a003c3e61eb445bb8df2",
    "blockNumber":"0x5daf3b",
    "from":"0xa7d9ddbe1f17865597fbd27ec712455208b6b76d",
    "gas":"0xc350",
    "gasPrice":"0x4a817c800",
    "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
    "input":"0x68656c6c6f21",
    "nonce":"0x15",
    "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
    "transactionIndex":"0x41",
    "value":"0xf3dbb76162000",
    "v":"0x25",
    "r":"0x1b5e176d927f8e9ab405058b2d2457392da3e20f328b16ddabcebc33eaac5fea",
    "s":"0x4ba69724e8f69de52f0125ad8b3c5c2cef33019bac3249e2c0a2192766d1721c"
  }"#,
        )
        .unwrap();

        let _res: Transaction = serde_json::from_str(
            r#"{
            "hash":"0xdd79ab0f996150aa3c9f135bbb9272cf0dedb830fafcbbf0c06020503565c44f",
            "nonce":"0xe",
            "blockHash":"0xef3fe1f532c3d8783a6257619bc123e9453aa8d6614e4cdb4cc8b9e1ed861404",
            "blockNumber":"0xf",
            "transactionIndex":"0x0",
            "from":"0x1b67b03cdccfae10a2d80e52d3d026dbe2960ad0",
            "to":"0x986ee0c8b91a58e490ee59718cca41056cf55f24",
            "value":"0x2710",
            "gas":"0x5208",
            "gasPrice":"0x186a0",
            "input":"0x",
            "v":"0x25",
            "r":"0x75188beb2f601bb8cf52ef89f92a6ba2bb7edcf8e3ccde90548cc99cbea30b1e",
            "s":"0xc0559a540f16d031f3404d5df2bb258084eee56ed1193d8b534bb6affdb3c2c"
    }"#,
        )
        .unwrap();
    }
}
