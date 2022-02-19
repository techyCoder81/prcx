use thiserror::Error;

pub use prc::*;

mod diff;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("A list in the result/patch is shorter than the corresponding list in the source data!")]
    ShortPatchList,
    #[error("A param must be the same type between the patch and the source data.")]
    NotSamePType,
    #[error("The resulting patch must be a struct!")]
    BadReturn
}

#[derive(PartialEq, Eq)]
enum ParamTag {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    Float,
    Hash,
    Str,
    List,
    Struct
}

trait GetPTag {
    fn get_tag(&self) -> ParamTag;
}

impl GetPTag for ParamKind {
    fn get_tag(&self) -> ParamTag {
        match self {
            ParamKind::Bool(_) => ParamTag::Bool,
            ParamKind::I8(_) => ParamTag::I8,
            ParamKind::U8(_) => ParamTag::U8,
            ParamKind::I16(_) => ParamTag::I16,
            ParamKind::U16(_) => ParamTag::U16,
            ParamKind::I32(_) => ParamTag::I32,
            ParamKind::U32(_) => ParamTag::U32,
            ParamKind::Float(_) => ParamTag::Float,
            ParamKind::Hash(_) => ParamTag::Hash,
            ParamKind::Str(_) => ParamTag::Str,
            ParamKind::List(_) => ParamTag::List,
            ParamKind::Struct(_) => ParamTag::Struct,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

// pub fn apply_patch(patch: &ParamStruct, source: &mut ParamStruct) -> Result<()> {

// }

pub fn generate_patch(source: &ParamStruct, result: &ParamStruct) -> Result<Option<ParamStruct>> {
    let param = diff::generate_diff_struct(source, result)?;
    if let Some(param) = param {
        if let ParamKind::Struct(s) = param {
            Ok(Some(s))
        } else {
            Err(Error::BadReturn)
        }
    } else {
        Ok(None)
    }
}

#[test]
fn generate_patch_test() {
    let result = open("./tests/hdr_fighter_param.prc").unwrap();
    let source = open("./tests/vanilla_fighter_param.prc").unwrap();
    let patch = generate_patch(&source, &result).unwrap().unwrap();
    save("./tests/hdr_fighter_paramx.prc", &patch).unwrap();
}

#[test]
fn generate_double_key_patch_test() {
    let result = open("./tests/stageparam_metroid_zebesdx_modded.stprm").unwrap();
    let source = open("./tests/stageparam_metroid_zebesdx.stprm").unwrap();
    let patch = generate_patch(&source, &result).unwrap().unwrap();
    save("./tests/stageparam_metroid_zebesdx.stprmx", &patch).unwrap();
}

#[test]
fn generate_added_entry_test() {
    let result = open("./tests/pictochat2_modded.stdat").unwrap();
    let source = open("./tests/pictochat2.stdat").unwrap();
    let patch = generate_patch(&source, &result).unwrap().unwrap();
    save("./tests/pictochat2.stdatx", &patch).unwrap();
}