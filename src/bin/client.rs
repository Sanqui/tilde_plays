extern crate byteorder;
extern crate nix;
extern crate users;

use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::str;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use nix::sys::termios;
use users::{get_user_by_uid, get_current_uid};

const MAGIC: u32 = 0x717DE;
const VERSION: u32 = 1;

const JOY_A: u32 = 1<<0;
const JOY_B: u32 = 1<<1;
const JOY_START: u32 = 1<<2;
const JOY_SELECT: u32 = 1<<3;

fn main() {
    println!("Tilde Plays client");
    
    let user = get_user_by_uid(get_current_uid()).unwrap();
    println!("Hello, ~{}!", user.name());
    
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
    
    if magic != MAGIC {
        panic!("Magic did not match, something else took over the port??")
    }
    if version != VERSION {
        panic!("Server version {} does not match client version {}", version, VERSION);
    }
    
    stream.write_u32::<BigEndian>(MAGIC).unwrap();
    stream.write_u32::<BigEndian>(VERSION).unwrap();
    
    let user_bytes = user.name().as_bytes();
    stream.write_u16::<BigEndian>(user_bytes.len() as u16).unwrap();
    stream.write(user_bytes).unwrap();
    
    println!("Connected!");
    
    // source: https://github.com/geofft/demo-rust-getch/blob/master/src/main.rs
    
    let saved_term = termios::tcgetattr(0).unwrap();
    let mut term = saved_term;
    
    term.c_lflag.remove(termios::ICANON);
    term.c_lflag.remove(termios::ISIG);
    term.c_lflag.remove(termios::ECHO);
    
    termios::tcsetattr(0, termios::TCSADRAIN, &term).unwrap();
    
    let keys = Arc::new(Mutex::new(Vec::new()));
    
    // input thread
    {
        let keys = keys.clone();  
        thread::spawn(move || {
            for byte in std::io::stdin().bytes() {
                let byte = byte.unwrap();
                let mut keys = keys.lock().unwrap();
                keys.push(byte as u32);
            }
        });
    }
    
    print!("{}[2J", 27 as char);
    
    
    let mut buttons = 0;
    'main: loop {
        let frame = stream.read_u32::<BigEndian>().unwrap();
        let screen_length = stream.read_u32::<BigEndian>().unwrap();
        
        let mut screen_buf: [u8; 4*160*144] = [0; 4*160*144];
        
        stream.read_exact(&mut screen_buf[0 .. screen_length as usize]).unwrap();
        let screen = str::from_utf8(&screen_buf[0..screen_length as usize]).unwrap();
        
        
        print!("{}[0;0H", 27 as char);
        println!("=== FRAME {} ===", frame);
        println!("{}", screen);
        
        println!("");
        println!("");
        
        buttons = 0;
        let mut keys = keys.lock().unwrap();
        for &key in keys.iter() {
            if key == 3 {
                break 'main;
            } else if key == 'z' as u32 {
                buttons |= JOY_A;
            } else if key == 'x' as u32 {
                buttons |= JOY_B;
            } else {
                print!("? {} ", key);
            }
        }
        print!("                          ");
        
        keys.drain(..);
        stream.write_u32::<BigEndian>(buttons).unwrap();
    }
    
    termios::tcsetattr(0, termios::TCSADRAIN, &saved_term).unwrap();
}



