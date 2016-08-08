extern crate byteorder;

use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::str;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

fn main() {
    println!("Tilde Plays client");
    
    let mut stream: TcpStream;
    
    println!("Connecting to manager...");
    loop {
        stream = match TcpStream::connect("127.0.0.1:13722") {
            Ok(stream) => stream,
            Err(err) => {
                println!("Failed to connect: {}", err);
                thread::sleep(Duration::from_millis(1000));
                continue
            }
        };
        break
    }
    
    let magic = stream.read_u32::<BigEndian>().unwrap();
    let version = stream.read_u32::<BigEndian>().unwrap();
    
    if magic != 0x717DE {
        panic!("Magic did not match, something else took over the port??")
    }
    if version != 1 {
        panic!("Server version does not match");
    }
    
    println!("Connected!");
    
    loop {
        let frame = stream.read_u32::<BigEndian>().unwrap();
        let screen_length = stream.read_u32::<BigEndian>().unwrap();
        
        let mut screen_buf: [u8; 4*160*144] = [0; 4*160*144];
        
        stream.read_exact(&mut screen_buf[0 .. screen_length as usize]).unwrap();
        let screen = str::from_utf8(&screen_buf[0..screen_length as usize]).unwrap();
        
        
        print!("{}[2J", 27 as char);
        println!("=== FRAME {} ===", frame);
        println!("{}", screen);
    }
}



