use std::path::Ancestors;

use super::executable::Executable;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ConfigUtil;

#[allow(dead_code)]
impl ConfigUtil {
    pub fn calc_id(paths: Vec<String>) -> usize {
        usize::from_be_bytes(ethers::utils::keccak256(paths.join(""))[..4].try_into().unwrap())
    }

    pub fn to_class_paths(name:String) -> Vec<String> {
        name.split(".").map(|part| part.replace("[i]", "")).collect::<Vec<_>>()      
    }
    pub fn to_instance_paths(name:String, ancestors: Vec<Executable>) -> Vec<String> {

        for e in ancestors {
            match e.mapping_key {
                Some(key) => {
                    // Bug: multiple keys in fullname and fail
                    // Dig belongs_to recursively and prepare keys beforehand
                    name.replace("[i]", &format!(".{}", key)).split(".").map(|part| part.to_string()).collect::<Vec<_>>()
                },
                None => {
                    // Note: fullname indicating corresponding amount of executables must be provided or should fail
                    name.split(".").map(|part| part.to_string()).collect::<Vec<_>>()
                }            
            }    
        }

    }

}
