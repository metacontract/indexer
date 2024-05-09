use std::path::Ancestors;

use super::executable::Executable;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ConfigUtil;

#[allow(dead_code)]
impl ConfigUtil {
    pub fn calc_id(paths: Vec<String>) -> usize {
        let path_string = paths.join("");
        let hash_bytes = ethers::utils::keccak256(path_string.as_bytes());
        let id_bytes: [u8; 4] = hash_bytes[..4].try_into().unwrap();
        let id = u32::from_be_bytes(id_bytes.try_into().unwrap());
        id as usize
    }

    pub fn to_class_paths(name:String) -> Vec<String> {
        name.split(".").map(|part| part.replace("[i]", "")).collect::<Vec<_>>()      
    }

}
