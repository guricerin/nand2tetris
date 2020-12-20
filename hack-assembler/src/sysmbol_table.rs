use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SymTableError {
    #[error("symbol <{0}> is already defined.")]
    AlreadyDefined(String),
    #[error("available address reach the upper limit.\tsymbol: {0}\taddress: {1}")]
    AddressLimit(String, Address),
}

pub type Address = u16;

const AVAILABLE_ADDRESS_END: Address = 0x4000;

pub struct SymbolTable {
    table: HashMap<String, Address>,
    /// 利用可能なアドレス
    vacant: Address,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut mp = HashMap::<String, Address>::new();

        // predefineds
        mp.insert("SP".to_owned(), 0);
        mp.insert("LCL".to_owned(), 1);
        mp.insert("ARG".to_owned(), 2);
        mp.insert("THIS".to_owned(), 3);
        mp.insert("THAT".to_owned(), 4);
        mp.insert("SCREEN".to_owned(), 0x4000);
        mp.insert("KBD".to_owned(), 0x6000);
        // R0 .. R15
        for i in 0..=15 {
            let s = format!("R{}", i);
            mp.insert(s, i);
        }

        Self {
            table: mp,
            vacant: 16,
        }
    }

    pub fn add_entry(&mut self, symbol: &str, address: Address) -> Result<(), SymTableError> {
        if self.vacant >= AVAILABLE_ADDRESS_END {
            return Err(SymTableError::AddressLimit(symbol.to_string(), self.vacant));
        }

        self.table.entry(symbol.to_string()).or_insert(address);
        self.vacant += 1;
        Ok(())
    }

    pub fn contains(&self, symbol: &str) -> bool {
        self.table.contains_key(&symbol.to_string())
    }

    pub fn get_address(&self, symbol: &str) -> Option<&Address> {
        self.table.get(&symbol.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// available address should be within [0x000f .. 0x3fff]
    #[test]
    fn test_add_entry_limit() {
        let mut table = SymbolTable::new();
        for i in 16..AVAILABLE_ADDRESS_END {
            let s = format!("a{}", i);
            let actual = table.add_entry(&s, i);
            assert!(actual.is_ok());
        }

        let actual = table.add_entry("hoge", AVAILABLE_ADDRESS_END);
        assert!(
            actual.is_err(),
            "available address should reach the upper limit"
        );
    }
}
