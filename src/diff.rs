use std::path::Path;

use prc::{
    ParamKind,
    ParamStruct,
    ParamList,
    hash40::{to_hash40, Hash40}
};

use serde::{
    Serialize,
    Deserialize
};

use crate::{
    key::{
        PrcKey,
        PrcKeyType
    }
};

#[derive(Serialize, Deserialize)]
pub struct Diff(pub Vec<(PrcKey, ParamKind)>);

impl Diff {
    fn find_key_in_list<'a>(key: &PrcKey, params: &'a mut ParamList) -> Option<&'a mut ParamKind> {
        let idx = match &key.ty {
            PrcKeyType::StructField(_) => return None,
            PrcKeyType::ListIndex(idx) => *idx
        };

        params.0
            .get_mut(idx)
            .map(|x| {
                if key.next.is_none() {
                    match x {
                        ParamKind::Struct(_) => None,
                        ParamKind::List(_) => None,
                        x => Some(x)
                    }
                } else {
                    match x {
                        ParamKind::Struct(s) if key.next.is_some() => {
                            Self::find_key_in_struct(key.next.as_ref().unwrap().as_ref(), s)
                        },
                        ParamKind::List(l) if key.next.is_some() => {
                            Self::find_key_in_list(key.next.as_ref().unwrap().as_ref(), l)
                        },
                        _ => None
                    }
                }
            })
            .flatten()
    }

    fn find_key_in_struct<'a>(key: &PrcKey, params: &'a mut ParamStruct) -> Option<&'a mut ParamKind> {
        let field = match &key.ty {
            PrcKeyType::StructField(hash) => *hash,
            PrcKeyType::ListIndex(_) => return None
        };

        let mut param = None;
        for (hash, p) in params.0.iter_mut() {
            if *hash == field {
                param = Some(p);
                break;
            }
        }

        param
            .map(|x| {
                if key.next.is_none() {
                    match x {
                        ParamKind::Struct(_) => None,
                        ParamKind::List(_) => None,
                        x => Some(x)
                    }
                } else {
                    match x {
                        ParamKind::Struct(s) if key.next.is_some() => {
                            Self::find_key_in_struct(key.next.as_ref().unwrap().as_ref(), s)
                        },
                        ParamKind::List(l) if key.next.is_some() => {
                            Self::find_key_in_list(key.next.as_ref().unwrap().as_ref(), l)
                        },
                        _ => None
                    }
                }
            })
            .flatten()
    }

    fn get_param_kind_from_str(s: &str) -> Option<ParamKind> {
        if s.starts_with("\"") && s.ends_with("\"") {
            Some(ParamKind::Str(s.trim_start_matches("\"").trim_end_matches("\"").to_string()))
        } else if s == "true" {
            Some(ParamKind::Bool(true))
        } else if s == "false" {
            Some(ParamKind::Bool(false))
        } else if s.starts_with("0x") {
            let s = s.trim_start_matches("0x");
            if let Ok(int) = i8::from_str_radix(s, 16) {
                Some(ParamKind::I8(int))
            } else if let Ok(int) = u8::from_str_radix(s, 16) {
                Some(ParamKind::U8(int))
            } else if let Ok(int) = i16::from_str_radix(s, 16) {
                Some(ParamKind::I16(int))
            } else if let Ok(int) = u16::from_str_radix(s, 16) {
                Some(ParamKind::U16(int))
            } else if let Ok(int) = i32::from_str_radix(s, 16) {
                Some(ParamKind::I32(int))
            } else if let Ok(int) = u32::from_str_radix(s, 16) {
                Some(ParamKind::U32(int))
            } else {
                Hash40::from_hex_str(s).ok().map(|x| ParamKind::Hash(x))
            }
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::I8(int))
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::U8(int))
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::I16(int))
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::U16(int))
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::I32(int))
        } else if let Ok(int) = s.parse() {
            Some(ParamKind::U32(int))
        } else if let Ok(float) = s.parse() {
            Some(ParamKind::Float(float))
        } else {
            Some(ParamKind::Hash(to_hash40(s)))
        }
    }

    fn find_diffs_in_struct(source: &ParamStruct, modded: &ParamStruct) -> Vec<(PrcKey, ParamKind)> {
        let mut vec = vec![];
        for (key, param) in source.0.iter() {
            let mut modded_param = None;
            for (m_key, m_param) in modded.0.iter() {
                if *m_key == *key {
                    modded_param = Some(m_param);
                }
            }

            if let Some (modded_param) = modded_param {
                if modded_param == param {
                    continue;
                }

                match param {
                    ParamKind::Struct(s) => {
                        if let ParamKind::Struct(s2) = modded_param {
                            vec.extend(
                                Self::find_diffs_in_struct(s, s2)
                                    .into_iter()
                                    .map(|(x, y)| (PrcKey {
                                        ty: PrcKeyType::StructField(*key),
                                        next: Some(Box::new(x))
                                    }, y))
                            );
                        } else {
                            unreachable!()
                        }
                    },
                    ParamKind::List(l) => {
                        if let ParamKind::List(l2) = modded_param {
                            vec.extend(
                                Self::find_diffs_in_list(l, l2)
                                    .into_iter()
                                    .map(|(x, y)| (PrcKey {
                                        ty: PrcKeyType::StructField(*key),
                                        next: Some(Box::new(x))
                                    }, y))
                            );
                        } else {
                            unreachable!()
                        }
                    },
                    _ => {
                        vec.push((PrcKey {
                            ty: PrcKeyType::StructField(*key),
                            next: None
                        }, modded_param.clone()))
                    }
                }
            }
        }

        vec
    }

    fn find_diffs_in_list(source: &ParamList, modded: &ParamList) -> Vec<(PrcKey, ParamKind)> {
        let mut vec = vec![];
        for (idx, param) in source.0.iter().enumerate() {
            if let Some(modded_param) = modded.0.get(idx) {
                if modded_param == param {
                    continue;
                }

                match param {
                    ParamKind::Struct(s) => {
                        if let ParamKind::Struct(s2) = modded_param {
                            vec.extend(
                                Self::find_diffs_in_struct(s, s2)
                                    .into_iter()
                                    .map(|(x, y)| (PrcKey {
                                        ty: PrcKeyType::ListIndex(idx),
                                        next: Some(Box::new(x))
                                    }, y))
                            );
                        } else {
                            unreachable!()
                        }
                    },
                    ParamKind::List(l) => {
                        if let ParamKind::List(l2) = modded_param {
                            vec.extend(
                                Self::find_diffs_in_list(l, l2)
                                    .into_iter()
                                    .map(|(x, y)| (PrcKey {
                                        ty: PrcKeyType::ListIndex(idx),
                                        next: Some(Box::new(x))
                                    }, y))
                            );
                        } else {
                            unreachable!()
                        }
                    },
                    _ => {
                        vec.push((PrcKey {
                            ty: PrcKeyType::ListIndex(idx),
                            next: None
                        }, modded_param.clone()))
                    }
                }
            }
        }

        vec
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        let patches = data
            .lines()
            .map(|x| {
                let splits = x.split("=").map(|x| x.trim()).collect::<Vec<&str>>();
                (splits[0], splits[1])
            })
            .filter_map(|(key, value)| {
                let key: PrcKey = key.parse().unwrap();
                Self::get_param_kind_from_str(value).map(|x| (key, x))
            })
            .collect();
        Ok(Self(patches))
    }

    pub fn open_bin<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        Ok(bincode::deserialize(&data).unwrap())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, is_text: bool) -> Result<(), std::io::Error> {
        if is_text {
            let mut file = std::fs::File::create(path)?;
            for (key, param) in self.0.iter() {
                use std::io::Write;
                match param {
                    ParamKind::Struct(_) => unreachable!(),
                    ParamKind::List(_) => unreachable!(),
                    ParamKind::Bool(bool) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), bool); },
                    ParamKind::I8(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },    
                    ParamKind::U8(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },
                    ParamKind::I16(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },    
                    ParamKind::U16(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },   
                    ParamKind::I32(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },    
                    ParamKind::U32(int) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), int); },   
                    ParamKind::Float(float) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), float); },   
                    ParamKind::Str(string) => { let _ = write!(&mut file, "{} = \"{}\"\n", key.to_string(), string); },   
                    ParamKind::Hash(hash) => { let _ = write!(&mut file, "{} = {}\n", key.to_string(), crate::hash::get(*hash)); },   
                }
            }
        } else {
            let data = bincode::serialize(self).unwrap();
            std::fs::write(path, &data)?;
        }
        Ok(())
    }

    pub fn apply(self, params: &mut ParamStruct) {
        for (key, value) in self.0 {
            if let Some(p) = Self::find_key_in_struct(&key, params) {
                *p = value;
            }
        }
    }

    pub fn generate(source: &ParamStruct, modded: &ParamStruct) -> Self {
        Self(Self::find_diffs_in_struct(source, modded))
    }
}

#[test]
fn write_diff() {
    let diffs = Diff(vec![
        ("fighter_param_table[0].landing_attack_air_frame_n".parse().unwrap(), ParamKind::Float(0.0)),
        ("fighter_param_table[0].landing_attack_air_frame_f".parse().unwrap(), ParamKind::Float(1.0)),
        ("fighter_param_table[0].landing_attack_air_frame_b".parse().unwrap(), ParamKind::Float(2.0)),
        ("fighter_param_table[0].landing_attack_air_frame_lw".parse().unwrap(), ParamKind::Float(3.0)),
        ("fighter_param_table[0].landing_attack_air_frame_hi".parse().unwrap(), ParamKind::Float(4.0)),
    ]);

    let data = bincode::serialize(&diffs).unwrap();
    std::fs::write("/home/blujay/dev/arc/prcx/fighter_param_patch.prcx", data).unwrap();
}

#[test]
fn read_diff() {
    hash::add_hashes(vec![
        "test",
        "test1",
        "test2",
        "test3",
        "test4",
        "test5",
        "test6",
    ]);
    let diffs = std::fs::read("/home/blujay/dev/arc/prcx/test.prcx").unwrap();
    let diffs: Diff = bincode::deserialize(&diffs).unwrap();
    let test_strs = vec![
        "test",
        "test1.test3",
        "test1.test4",
        "test2[0].test5",
        "test2[1].test6",
    ];
    for (idx, (path, prm)) in diffs.0.iter().enumerate() {
        if let ParamKind::Float(f) = prm {
            assert_eq!(*f, 10.0);
        } else {
            panic!("ParamKind was not float!");
        }
        assert_eq!(path.to_string(), test_strs[idx]);
    }
}

#[test]
fn apply_diff() {
    let diffs = Diff(vec![
        ("fighter_param_table[0].walk_accel_mul".parse().unwrap(), ParamKind::Float(100.0))
    ]);
    let mut params = prc::open("/home/blujay/dev/arc/prcx/fighter_param.prc").unwrap();
    diffs.apply(&mut params);
    prc::save("/home/blujay/dev/arc/prcx/fighter_param_out.prc", &params).unwrap();
}

#[test]
fn read_and_apply_bin_diff() {
    let diffs = std::fs::read("/home/blujay/dev/arc/prcx/fighter_param_patch.prcx").unwrap();
    let diffs: Diff = bincode::deserialize(&diffs).unwrap();
    let mut params = prc::open("/home/blujay/dev/arc/prcx/fighter_param.prc").unwrap();
    diffs.apply(&mut params);
    prc::save("/home/blujay/dev/arc/prcx/fighter_param_out.prc", &params).unwrap();
}

#[test]
fn read_and_apply_text_diff() {
    let diffs = Diff::open("/home/blujay/dev/arc/prcx/fighter_param_patch.prctxt").unwrap();
    let mut params = prc::open("/home/blujay/dev/arc/prcx/fighter_param.prc").unwrap();
    diffs.apply(&mut params);
    prc::save("/home/blujay/dev/arc/prcx/fighter_param_out.prc", &params).unwrap();
}