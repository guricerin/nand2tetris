use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SymTableError {
    #[error("<{0}> is already defined.")]
    AlreadyDefined(String),
    #[error("available address reach the upper limit.\tsymbol: {0}\taddress: {1}")]
    AddressLimit(String, Address),
}

pub type Address = u16;

const AVAILABLE_ADDRESS_LIMIT: Address = 0x4000;

pub struct SymbolTable {
    table: BTreeMap<String, Address>,
    /// 利用可能なアドレス
    vacant: Address,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut mp = BTreeMap::<String, Address>::new();

        // predefineds
        mp.insert("SP".to_owned(), 0);
        mp.insert("LCL".to_owned(), 1);
        mp.insert("ARG".to_owned(), 2);
        mp.insert("THIS".to_owned(), 3);
        mp.insert("THAT".to_owned(), 4);
        mp.insert("SCREEN".to_owned(), 16384);
        mp.insert("KBD".to_owned(), 24576);
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
        if self.vacant >= AVAILABLE_ADDRESS_LIMIT {
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
