extern crate byteorder;

use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::thread;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const BPP: usize = 4;
const NUMTILES: usize = WIDTH/8*HEIGHT/8;

type Tile = [u8; 8*8];
type ScreenTiles = [Tile; NUMTILES];

fn tilify(screen: [u8; WIDTH*HEIGHT*BPP]) -> ScreenTiles {
    let mut tiles: ScreenTiles = [[0; 8*8]; NUMTILES];
    
    for tilerow in 0..HEIGHT/8 {
        for row in 0..8 {
            for tilecol in 0..WIDTH/8 {
                for col in 0..8 {
                    let screenoff = ((tilerow*8+row)*WIDTH + tilecol*8+col)*BPP;
                    let pixelcolor = screen[screenoff] as u8;
                    tiles[tilerow*WIDTH/8 + tilecol][row*8+col] =
                        match pixelcolor {
                            0xf8 => 0,
                            0xa8 => 1,
                            0x50 => 2,
                            0x00 => 3,
                            _ => panic!("Unexpected color value")
                        };
                }
            }
        }
    }
    
    tiles
}

fn tile_to_ascii(tile: Tile) -> char {
    //let sum = tile.iter().sum();
    // doesn't work, sums into an u8, too small
    
    let mut sum: u32 = 0;
    for val in tile.iter() {
        sum += *val as u32;
    }
    let max = 3*8*8;
    
    
    if      sum == 0      { ' ' }
    else if sum < max*1/4 { '.' }
    else if sum < max*2/4 { 'o' }
    else if sum < max*3/4 { 'O' }
    else if sum < max     { '%' }
    else if sum == max    { '@' }
    else { panic!("Impossible tile sum") }
}

fn tiles_to_ascii(screen: ScreenTiles) -> String {
    let mut output = String::with_capacity(NUMTILES+HEIGHT);
    
    let mut col = 0;
    for tile in screen.iter() {
        output.push(tile_to_ascii(*tile));
        col += 1;
        if col == WIDTH/8 {
            output.push('\n');
            col = 0;
        }
    }
    
    output
}

fn main() {
    println!("Tilde Plays manager");
    
    let mut stream = TcpStream::connect("127.0.0.1:13721").unwrap();
    
    
    let width = stream.read_u32::<BigEndian>().unwrap();
    let height = stream.read_u32::<BigEndian>().unwrap();
    let bpp = stream.read_u32::<BigEndian>().unwrap();
    
    println!("Resolution: {} x {} @ {} bytes per pixel", width, height, bpp);
    
    if width != WIDTH as u32 || height != HEIGHT as u32 || bpp != BPP as u32 {
        println!("Invalid dimensions");
        return;
    }
    
    let mut frame = 0;
    
    loop {
        //println!("Frame {}", frame);
        
        let mut buttons = 0;
        
        stream.write_u16::<BigEndian>(buttons).unwrap();
        
        //let mut screen: Vec<u8> = vec![0; (width*height*bpp) as usize];
        let mut screen: [u8; WIDTH*HEIGHT*BPP] = [0; WIDTH*HEIGHT*BPP];
        
        stream.read_exact(&mut screen).unwrap();
        
        //println!("read {} bytes", screen.len());
        let screen_ascii = tiles_to_ascii(tilify(screen));
        print!("{}[2J", 27 as char);
        println!("=== FRAME {} ===", frame);
        println!("{}", screen_ascii);
        frame += 1;
        //thread::sleep(Duration::from_millis(25));
    }
}



