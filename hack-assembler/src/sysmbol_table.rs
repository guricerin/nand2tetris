use super::types::Address;
use crate::parser::command::*;
use crate::parser::common::*;

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SymTableError {
    #[error("symbol <{0}> is already defined.")]
    AlreadyDefined(String),
    #[error("available address reach the upper limit.")]
    AddressLimit,
}

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

    pub fn get_address(&self, symbol: &str) -> Option<&Address> {
        self.table.get(&symbol.to_string())
    }

    pub fn resolve(&mut self, commands: &Vec<Command>) -> Result<(), SymTableError> {
        let mut line_num: Address = 0;
        for c in commands.iter() {
            match c {
                Annot {
                    value: CommandKind::L(LabelCommand { label, .. }),
                    ..
                } => {
                    self.add_label(&label, line_num)?;
                }
                _ => {
                    line_num += 1;
                }
            }
        }
        for c in commands.iter() {
            match c {
                Annot {
                    value:
                        CommandKind::A(AddrCommand {
                            value: NumOrSymbol::Symbol(symbol),
                            ..
                        }),
                    ..
                } => {
                    self.add_symbol(&symbol)?;
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn check_address_limit(&self) -> Result<(), SymTableError> {
        if self.vacant >= AVAILABLE_ADDRESS_END {
            Err(SymTableError::AddressLimit)
        } else {
            Ok(())
        }
    }

    fn add_symbol(&mut self, symbol: &str) -> Result<(), SymTableError> {
        self.check_address_limit()?;

        let vacant = &mut self.vacant;
        self.table.entry(symbol.to_string()).or_insert_with(|| {
            let address = vacant.clone();
            *vacant += 1;
            address
        });
        Ok(())
    }

    fn add_label(&mut self, label: &str, address: Address) -> Result<(), SymTableError> {
        self.check_address_limit()?;

        self.table.entry(label.to_string()).or_insert(address);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry_limit() {
        let mut table = SymbolTable::new();
        for i in 16..AVAILABLE_ADDRESS_END {
            let s = format!("a{}", i);
            let actual = table.add_symbol(&s);
            assert!(actual.is_ok());
        }

        let actual = table.add_symbol("hoge");
        assert!(actual.is_err(), "available address reached the upper limit");
    }
}
