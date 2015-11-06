use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::env::args;
static NEWLINE: u8 = '\n' as u8; 
static NUMBERSIGN: u8 = '#' as u8; 
static WHITESPACE: char = ' '; 

fn file_to_bytevec(input_file: &String) -> Result<Vec<u8>, io::Error> {
    let file = try!(File::open(&input_file)); 
    let bytes = file.bytes();
    Ok(try!(bytes.collect()))
}

/* FIXME: Rudimentaerer PGM-Parser: 
  - Kommentare werden nach der depth-Line nicht mehr als Solche erkannt
  - Kommentare werde allgemein nur erkannt, wenn die ganze Zeile mit # beginnt
  - Als Trennzeichen wird nur Newline erkannt, die PGM-spec erlaubt auch andere
  - Ausnahme ist die dimension-zeile, dort stehen sowohl width als auch height,
    getrennt durch ein Whitespace
  - PGM erlaubt die Nutzdaten auch als ASCII zu speichern (wohingegen der 
    Header immer ASCII ist?), hier werden die Daten nur binaer akzeptiert
  - Parser basiert auf die PGM-Files, die GIMP exportieren kann
*/
fn parse_pgm(bytes: &Vec<u8>) -> Result<(usize, usize, usize, usize), io::Error>  {
    let (x, y) = try!(get_next_noncomment_line(&bytes,0 , bytes.len()));
    let magic: String = u8_slice_to_string(&bytes[x..y]);
    if magic != "P5" {
        let msg = format!("Unknown magic number {}", magic);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, msg));
    }
    let (x, y) = try!(get_next_noncomment_line(&bytes, y+1, bytes.len()));
    let dimension: String = u8_slice_to_string(&bytes[x..y]);
    let (x, y) = try!(get_next_noncomment_line(&bytes, y+1, bytes.len()));
    let depth = match u8_slice_to_string(&bytes[x..y]).parse::<usize>() {
        Ok(x) => x,
        Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                            "Could not parse depth")),
    };
    let (width, height) = try!(parse_dimension(dimension));
    Ok((y+1, depth, width, height))
}

fn parse_dimension(dimension: String) -> Result<(usize, usize), io::Error> {
    let split_pos = match dimension.chars().position(|r| r == WHITESPACE) {
        Some(x) => x,
        None    => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                             "Could not parse dimensions")),
    };
    let width = match dimension[0..split_pos].parse::<usize>() {
        Ok(x) => x,
        Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                             "Could not parse dimensions")),
    };
    let height = match dimension[split_pos+1..].parse::<usize>() {
        Ok(x) => x,
        Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                             "Could not parse dimensions")),
    };
    Ok((width,height))
}

fn u8_slice_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&x| x as char).collect()
}

fn get_next_noncomment_line(bytes: &[u8], first: usize, last: usize) 
    -> Result<(usize,usize),io::Error> {
    let vec = &bytes[first..last];
    let mut start = 0;
    let mut end;
    loop {
      end = match vec[start..].iter().position(|&r| r == NEWLINE) {
          Some(x) => x,
          None => return Err(io::Error::new(
              io::ErrorKind::InvalidInput,
              "No newline found")),
      };
      if vec[start] != NUMBERSIGN {
          break;
      }
      start = end + 1;
      if start >= vec.len() {
        return Err(io::Error::new(
              io::ErrorKind::InvalidInput,
              "Vec size exceeded"));
      }
    }
    Ok((start+first,end+start+first))
}

unsafe fn get_min_max(data:&[u8], width: usize, height: usize) -> (f64, f64 ) {
    let mut min = std::f64::MAX;
    let mut max = std::f64::MIN;
    for i in width + 1..((height * width)-width) {
        if !(i % width   == 0 || 
            (i+1) % width == 0) {
            let x:f64 = sobel(data, i, width);
            if x < min {
                min = x;
            } else if x > max { 
                max = x;
            }
        }
    }
    (min, max)
}

unsafe fn sobel(data: &[u8], pos: usize, width: usize) -> f64 {
    ((sobel_y(data, pos, width).abs() + sobel_x(data, pos, width).abs()) as f64).sqrt()
}

unsafe fn sobel_y(data: &[u8], pos: usize, width: usize) -> i64 {
    (-(*data.get_unchecked((pos - width - 1)) as i64 ) +  -((*data.get_unchecked((pos - width)) as i64) << 2) + 
     -(*data.get_unchecked((pos - width + 1)) as i64 ) + (*data.get_unchecked((pos + width - 1))  as i64) +
     ((*data.get_unchecked((pos + width)) as i64) << 2)      + (*data.get_unchecked((pos + width + 1)) as i64))
}

unsafe fn sobel_x(data: &[u8], pos: usize, width: usize) -> i64 {
    ((*data.get_unchecked((pos - width - 1))  as i64)) + -(*data.get_unchecked((pos - width + 1)) as i64) + 
    ((*data.get_unchecked(pos - 1)            as i64) << 1) + -((*data.get_unchecked((pos + 1))         as i64)  << 1) + 
    (*data.get_unchecked((pos + width - 1))  as i64) + -(*data.get_unchecked((pos + width + 1)) as i64)
}

/*
unsafe fn sobel(data: &[u8], pos: usize, width: usize) -> f64 {
    ((sobel_y(data, pos, width).abs() + sobel_x(data, pos, width).abs())).sqrt()
}

unsafe fn sobel_y(data: &[u8], pos: usize, width: usize) -> f64 {
    ((*data.get_unchecked((pos - width - 1)) as f64 * (-1.0)) + (*data.get_unchecked((pos - width)) as f64 * (-2.0)) + 
     (*data.get_unchecked((pos - width + 1)) as f64 * (-1.0)) + (*data.get_unchecked((pos + width - 1))  as f64 * (1.0)) +
     (*data.get_unchecked((pos + width)) as f64 * (2.0))      + (*data.get_unchecked((pos + width + 1)) as f64 * (1.0)))
}

unsafe fn sobel_x(data: &[u8], pos: usize, width: usize) -> f64 {
    ((*data.get_unchecked((pos - width - 1))  as f64 * (1.0))) + (*data.get_unchecked((pos - width + 1)) as f64 * (-1.0)) + 
    (*data.get_unchecked(pos - 1)            as f64 * (2.0)) + (*data.get_unchecked((pos + 1))         as f64 * (-2.0)) + 
    (*data.get_unchecked((pos + width - 1))  as f64 * (1.0)) + (*data.get_unchecked((pos + width + 1)) as f64 * (-1.0))
}
*/

fn scale(val_to_scale: f64, min: f64, max: f64, scaled_max: f64) -> f64 {
  (scaled_max * ((val_to_scale - min) / (max - min)))
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 3 {
        panic!("Usage: {} in.pgm out.pgm", args[0] );
    }
    let vec = match file_to_bytevec(&args[1]) {
        Ok(x) => x,
        Err(why) => panic!("couldn't open {}: {}", &args[2],
                                                   why.description()),
    };
    let (data_start, depth, width, height) = match parse_pgm(&vec) {
        Ok((data_start, depth, width, height)) => (data_start, depth, width, height),
        Err(why) => panic!("panic! {}", why),
    };
    let header = &vec[0 .. data_start];
    let read_data = &vec[data_start..];
    let mut write_data = vec![0; (vec.len() - header.len())];
    let mut outfile = match File::create(&args[2]) {
        Ok(file) => file,
        Err(why) => panic!("couldn't open {}: {}", &args[2],
                                                   why.description()),
    };
    if let Err(why) = outfile.write_all(header) {
        panic!("couldn't write to {}: {}", &args[2], why.description())
    }
    unsafe {
        let (min, max) = get_min_max(read_data, width, height);
        for i in width+1..((height * width) - width) {
            if !(i % width   == 0 || 
                (i + 1) % width == 0) {
                let x:f64 = sobel(read_data, i, width);
                write_data[i] = scale(x, min, max, depth as f64) as u8;
            }
        } 
    }
      
    if let Err(why) = outfile.write_all(&write_data) {
        panic!("couldn't write to {}: {}", &args[2], why.description())
    }
}
