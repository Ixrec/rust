// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Test performance of a thread "spawn ladder", in which children thread have
// many ancestor threadgroups, but with only a few such groups alive at a time.
// Each child thread has to enlist as a descendant in each of its ancestor
// groups, but that shouldn't have to happen for already-dead groups.
//
// The filename is a song reference; google it in quotes.

// ignore-pretty very bad with line comments

use std::sync::mpsc::{channel, Sender};
use std::env;
use std::thread;

fn child_generation(gens_left: usize, tx: Sender<()>) {
    // This used to be O(n^2) in the number of generations that ever existed.
    // With this code, only as many generations are alive at a time as threads
    // alive at a time,
    thread::spawn(move|| {
        if gens_left & 1 == 1 {
            thread::yield_now(); // shake things up a bit
        }
        if gens_left > 0 {
            child_generation(gens_left - 1, tx); // recurse
        } else {
            tx.send(()).unwrap()
        }
    });
}

fn main() {
    let args = env::args();
    let args = if env::var_os("RUST_BENCH").is_some() {
        vec!("".to_string(), "100000".to_string())
    } else if args.len() <= 1 {
        vec!("".to_string(), "100".to_string())
    } else {
        args.collect()
    };

    let (tx, rx) = channel();
    child_generation(args[1].parse().unwrap(), tx);
    if rx.recv().is_err() {
        panic!("it happened when we slumbered");
    }
}