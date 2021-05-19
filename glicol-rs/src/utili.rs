extern crate lcs_diff;
use self::lcs_diff::*;
use super::{EngineError};
// use regex::Regex;

pub fn midi_or_float(num: String) -> f32 {
    if num.contains(".") {
        num.parse::<f32>().unwrap()
    } else {
        let midi = num.parse::<f32>().unwrap();
        if midi == 0.0 {
            0.0
        } else {
            2.0f32.powf((midi - 69.0)/12.0) * 440.0
        }
    }
}

pub fn preprocess_sin(a: &String) -> Result<String, EngineError> {
    let q: String = a.replace("\n", " \n");
    let v: Vec<&str> = q.split(" ").collect();
    // println!("{:?}", v);
    let mut b = "".to_string();
    let mut skip = false;
    for (i, c) in v.iter().enumerate() {
        if *c == "sin" || *c == "saw" || *c == "squ" {
            if v[i+1].parse::<f32>().is_ok() {
                b += "const ";
                b += v[i+1];
                b += " >> ";
                b += c;
                skip = true;
            } else {
                b += c;
                b += " ";
            }
            // println!("{:?} {:?}", i, c);
        } else {
            if skip {
                b += " 1 ";
                skip = false;
            } else {
                b += c;
                b += " ";
            }
        }
    }
    Ok(b)
}

pub fn preprocess_mul(a: &String) -> Result<String, EngineError> {
    let q: String = a.replace("\n", " \n ");
    let v: Vec<&str> = q.split(" ").collect();
    // println!("{:?}", v);
    let mut b = "".to_string();
    let mut current_ref = "";
    let x = "abcdefghijklmnopqrstuvwxyz".to_string();
    let mut append = Vec::<(&str, &str, &str)>::new();
    let mut find = false;
    let mut index:usize = 0;
    for (i, c) in v.iter().enumerate() {
        if c.contains(":") {
            current_ref = &c[1..c.len()-1];
            index = 0;
            b += c;
            b += " ";
        } else if c == &"mul" || c == &"*" {
            if v[i+1].parse::<f32>().is_ok() {
                append.push((current_ref, &x[index..(index+1)], v[i+1]));
                find = true;
            };
            b += c;
            b += " ";
        } else if find == true {
            let s = format!("~{}mulconst{}",
            append.last().unwrap().0, append.last().unwrap().1);
            b += &s;
            b += " ";
            find = false;
            index += 1;
        } else if c == &"\n" {
            b += c;
        } else {
            b += c;
            b += " ";
        }
        
    }
    for x in append {
        b += &format!("\n\n~{}mulconst{}: const {}", x.0, x.1, x.2);
    }
    Ok(b)
}

pub fn lcs(old: &Vec<String>, new: &Vec<String>)
-> (Vec<(String, usize)>, Vec<(String, usize)>, Vec<String>) {
    
    let mut add = Vec::new();
    let mut rem = Vec::new();
    let mut del = Vec::new();
    for diff in lcs_diff::diff(&old, &new) {
        // println!("\n\nold {:?}, new {:?}\n\n", &old, &new);
        match diff {
            DiffResult::Added(a) => {
                add.push((a.data.clone(), a.new_index.unwrap()));
                // println!(
                //     "+{} new index = {}",
                //     a.data,
                //     a.new_index.unwrap()
                // );                       
            },
            DiffResult::Common(c) => {
                rem.push((c.data.clone(), c.new_index.unwrap()));
                // println!(
                //     "{} old index = {}, new index = {}",
                //     c.data,
                //     c.old_index.unwrap(),
                //     c.new_index.unwrap()
                // );
            },
            DiffResult::Removed(r) => {
                // println!("to remove!!! {:?}", r);
                del.push(r.data.clone());
                // println!(
                //     "-{} old index = {}",
                //     r.data,
                //     r.old_index.unwrap()
                // );
            }
        }
    };
    // println!("\n{:?}{:?}{:?}\n", add, rem, del);
    (add, rem, del)
}

// pub fn clamp(input: f32, min: f32, max: f32) -> f32 {
//     match input {
//         c if c < min => min,
//         c if c > max => max,
//         _ => input
//     }
// }

pub fn process_error_info(code: String, error: usize, s: usize, e: usize) -> [u8; 256] {
    let mut info: [u8; 256] = [0; 256];
    println!("{:?} {:?}", code, s);
    let line = code[..s].matches("\n").count() as u8;
    info[0] = error as u8;
    info[1] = line;
    let word = code[s..e].as_bytes();
    if word.len() < 254 {
        for i in 2..word.len()+2 {
            info[i] = word[i-2]
        }
    } else {
        for i in 2..256 {
            info[i] = word[i-2]
        }
    }
    info
}