use std::{str::FromStr, cmp::Ordering};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use prc::{hash40::{Hash40, to_hash40}};

use crate::hash;

#[derive(Debug, Serialize, Deserialize)]
pub enum PrcKeyType {
    StructField(Hash40),
    ListIndex(usize)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrcKey {
    pub ty: PrcKeyType,
    pub next: Option<Box<PrcKey>>
}

#[derive(Error, Debug)]
pub enum PrcKeyError {
    #[error("The PRC key was malformed.")]
    Malformed,
}

impl PrcKey {
    const INVALID_STR_INDEX: usize = 0x300;

    fn get_key(index: &str) -> Option<Self> {
        // If the key starts with "." we want to ignore it, i.e. for ".some_struct.some_key"
        // we only want to worry about "some_struct.some_key"
        let index = if index.starts_with(".") {
            index.split_at(1).1
        } else {
            index
        };

        // Get out if the index str is empty
        if index == "" {
            return None;
        }

        // If we find "." before "[", it means that this first key is a struct and not a list
        let dot_index = index.find(".").unwrap_or(Self::INVALID_STR_INDEX);
        let bracket_index = index.find("[").unwrap_or(Self::INVALID_STR_INDEX);

        match dot_index.cmp(&bracket_index) {
            Ordering::Equal => Some(PrcKey {
                ty: PrcKeyType::StructField(to_hash40(index)),
                next: None
            }),
            Ordering::Less => {
                // Split the parent key from the next key and hash it
                let (parent_key, next_index) = index.split_at(dot_index);
                let parent_key = to_hash40(parent_key);

                // Return a new PrcKey with the parent key name and the parsed index of the next key
                Some(PrcKey {
                    ty: PrcKeyType::StructField(parent_key),
                    next: Self::get_key(next_index).map(|x| Box::new(x))
                })
            },
            Ordering::Greater => {
                // Split the parent key from the index
                let (parent_key, list_idx) = index.split_at(bracket_index);
                // Extract the numerical index from inside of the array index
                let list_end = list_idx.find("]").expect("List index was not terminated!");
                let (list_key, next_key) = list_idx.split_at(list_end + 1);
                let list_idx = list_key.trim_start_matches("[").trim_end_matches("]").parse::<usize>().expect("List index was malformed!");
                // Create the list index prc key
                let result = PrcKey {
                    ty: PrcKeyType::ListIndex(list_idx),
                    next: Self::get_key(next_key).map(|x| Box::new(x))
                };
                // Check if the parent key is not empty, if it isn't then add wrap the index key inside of the parent key
                if parent_key == "" {
                    Some(result)
                } else {
                    let parent_key = to_hash40(parent_key);
                    Some(PrcKey {
                        ty: PrcKeyType::StructField(parent_key),
                        next: Some(Box::new(result))
                    })
                }
            }
        }
    }

    fn write_str(&self, hashed: bool) -> String {
        let current = self.next.as_ref().map(|x| x.write_str(hashed)).unwrap_or("".to_string());
        match self.ty {
            PrcKeyType::StructField(name) => {
                // If we want the field names to stay hashed then we need to just format the hex
                let name = if hashed {
                    format!("{:#x}", name.0)
                } else {
                    hash::get(name)
                };
                format!(".{}{}", name, current)
            },
            PrcKeyType::ListIndex(idx) => {
                // Format an index like an array index operator
                format!("[{}]{}", idx, current)
            }
        }
    }

    // like to_string however it allows arguments to stay hashed if that's preferred
    fn to_str(&self, hashed: bool) -> String {
        self.write_str(hashed).trim_start_matches(".").to_string()
    }
}

impl FromStr for PrcKey {
    type Err = PrcKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::get_key(s) {
            Some(key) => Ok(key),
            None => Err(PrcKeyError::Malformed)
        }
    }
}

impl ToString for PrcKey {
    fn to_string(&self) -> String {
        self.write_str(false).trim_start_matches(".").to_string()
    }
}

#[test]
fn hash_test() {
    let key: PrcKey = match "test_struct.test_field.test_field_2".parse() {
        Ok(key) => key,
        Err(e) => panic!("Failed to parse key: {:?}", e)
    };
    
    let test = format!("{:#x}.{:#x}.{:#x}", to_hash40("test_struct").0, to_hash40("test_field").0, to_hash40("test_field_2").0);
    assert_eq!(test, key.to_str(true));
}

#[test]
fn unhash_test() {
    hash::add_hashes(vec![
        "test_struct",
        "test_field",
        "test_field_2"
    ]);
    let key: PrcKey = match "test_struct.test_field.test_field_2".parse() {
        Ok(key) => key,
        Err(e) => panic!("Failed to parse key: {:?}", e)
    };

    assert_eq!("test_struct.test_field.test_field_2", key.to_string());
}

#[test]
fn unhash_list_test() {
    hash::add_hashes(vec![
        "test_list",
        "test_struct",
        "test_field",
        "test_field_2"
    ]);
    let key: PrcKey = match "test_list[1].test_struct.test_field[3].test_field_2".parse() {
        Ok(key) => key,
        Err(e) => panic!("Failed to parse key: {:?}", e)
    };

    assert_eq!("test_list[1].test_struct.test_field[3].test_field_2", key.to_string());
}

#[test]
fn hash_list_test() {
    let key: PrcKey = match "test_list[1].test_struct.test_field[3].test_field_2".parse() {
        Ok(key) => key,
        Err(e) => panic!("Failed to parse key: {:?}", e)
    };
    
    let test = format!("{:#x}[1].{:#x}.{:#x}[3].{:#x}", to_hash40("test_list").0, to_hash40("test_struct").0, to_hash40("test_field").0, to_hash40("test_field_2").0);
    assert_eq!(test, key.to_str(true));
}
