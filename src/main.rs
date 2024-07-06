#![allow(dead_code)]

use std::{
    collections::VecDeque,
    fs::File,
    io::{stdin, stdout, Read, Write}, time::Instant,
};

use um::{IOInterface, UniversalMachine};

mod um;

fn codex() -> Vec<u8> {
    let mut f = File::open("data/codex.umz").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

fn sandmark() -> Vec<u8> {
    let mut f = File::open("data/sandmark.umz").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

struct UMIO {
    input_buffer: VecDeque<u8>,
}

impl UMIO {
    fn new(s: &str) -> Self {
        return UMIO {
            input_buffer: s.chars().map(|c| c as u8).collect(),
        };
    }
}

impl IOInterface for UMIO {
    fn request_input(&mut self) -> u8 {
        eprintln!("Request input!");
        match self.input_buffer.pop_front() {
            Some(c) => c,
            None => {
                let mut stdin_handle = stdin().lock();
                let mut byte = [0_u8];
                stdin_handle.read_exact(&mut byte).unwrap();
                byte[0]
            }
        }
    }

    fn request_output(&mut self, ch: u8) {
        stdout().lock().write(&[ch]).unwrap();
    }
}

fn run_codex() {
    let mut io = UMIO::new(r"(\b.bb)(\v.vv)06FHPVboundvarHRAk");
    let mut um = UniversalMachine::new(&codex(), &mut io).unwrap();
    um.run(None);
}

fn run_sandmark() {
    let mut io = UMIO::new(r"");
    let mut um = UniversalMachine::new(&sandmark(), &mut io).unwrap();
    um.run(None);
}

fn main() {
    run_sandmark();
}

#[test]
fn bench() {
    let t = Instant::now();
    let mut io = UMIO::new(r"");
    let mut um = UniversalMachine::new(&sandmark(), &mut io).unwrap();
    um.run(Some(100_000_000));
    eprintln!("Elapsed: {:?}", t.elapsed());
}