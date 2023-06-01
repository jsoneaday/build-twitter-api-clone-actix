use std::io::Read;
use std::fs::{File, metadata};

pub fn get_avatar_buffer(file_path: &str) -> Vec<u8> {    
    let mut avatar = File::open(file_path).expect("Profile file was not found");
    let metadata = metadata(file_path).expect("Profile file metadata not found");
    let mut avatar_buffer = vec![0; metadata.len() as usize];
    avatar.read(&mut avatar_buffer).expect("Buffer overflow");
    avatar_buffer
}