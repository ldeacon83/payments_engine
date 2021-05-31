use std::{cmp::Ordering};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TransactionType {
    deposit,
    withdrawal,
    dispute,
    resolve,
    chargeback
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Transaction {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: u16,
    amount: Option<f64>,
    tx: u32,
}

impl Transaction {
    #[allow(dead_code)]
    pub fn from(tx_type: TransactionType,
                client: u16,
                tx: u32,
                amount: Option<f64>) -> Self {
                    Self{tx_type, client, amount, tx}
                }

    pub fn amount(&self) -> Option<f64>{
        self.amount
    }

    pub fn tx_type(&self) -> &TransactionType {
        &self.tx_type
    }

    pub fn client(&self) -> &u16 {
        &self.client
    }

    pub fn tx(&self) -> &u32 {
        &self.tx
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self)-> Ordering {
        self.tx.cmp(&other.tx)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        (self.tx_type == other.tx_type) && 
        (self.client == other.client) &&
        (self.amount == other.amount) &&
        (self.tx == other.tx)
    }
}

impl Eq for Transaction { }

#[cfg(test)]
mod tests {

   use super::*;
    use std::collections::BTreeSet;

    const INPUT: &str = "type,client,tx,amount\ndeposit,1,1,1.0\nwithdrawal,2,2,2.0\ndispute,1,2,\nresolve,1,2,\nchargeback,1,2,\n";

    #[test]
    fn test_csv() {
        let mut rdr = csv::Reader::from_reader(INPUT.as_bytes());
        let mut expected = BTreeSet::<Transaction>::new();
        expected.insert(Transaction{tx_type:TransactionType::deposit,client: 1, tx: 1, amount: Some(0.0)});
        expected.insert(Transaction{tx_type:TransactionType::withdrawal,client: 2, tx: 2, amount: Some(2.0)});
        expected.insert(Transaction{tx_type:TransactionType::dispute,client: 1, tx: 2, amount: None});
        expected.insert(Transaction{tx_type:TransactionType::resolve,client: 1, tx: 2, amount: None});
        expected.insert(Transaction{tx_type:TransactionType::chargeback,client: 1, tx: 2, amount: None});
        let mut actual = BTreeSet::<Transaction>::new();
        for result in rdr.deserialize() {
            actual.insert(result.unwrap());
        }
    }
}