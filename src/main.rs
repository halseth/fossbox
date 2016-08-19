extern crate clap;
extern crate notify;

use clap::{Arg, App};
use notify::{RecommendedWatcher, Watcher};
use std::sync::mpsc::channel;
use std::process::Command;

// Watch a directory for changes
fn watch(directory: &str) -> notify::Result<()> {
  let (tx, rx) = channel();
  let mut watcher: RecommendedWatcher = try!(Watcher::new(tx));
  try!(watcher.watch(directory));

  println!("Watching {:?}", directory);

  // Keep track of the hash of the directory the last time we published it to IPNS
  let mut published_hash = String::new();

  loop {
      match rx.recv() {
        Ok(notify::Event{ path: Some(path),op:Ok(op) }) => {
            println!("{:?} {:?}", op, path);
            let hash = ipfs_add_dir(directory);
            published_hash = ipns_publish_dir(&hash, &published_hash)
        },
        Err(e) => println!("watch error {}", e),
        _ => ()
      }
  }
}

// Add directory to ipfs. Returns hash of the added directory
fn ipfs_add_dir(directory: &str) -> String {
    println!("Adding directory: {:?}", directory);
    let output = Command::new("ipfs")
                    .arg("add")
                    .arg("-r")
                    .arg(directory)
                    .output()
                    .expect("ipfs add failed");

    let out_string = String::from_utf8_lossy(&output.stdout);

    // Second to last string will be hash of the directory
    let directory_hash = out_string.split_whitespace().rev().nth(1).unwrap();

    println!("Directory added with hash {:?}", directory_hash);
    directory_hash.to_string()
}

// Publish directory hash to ipns
fn ipns_publish_dir(directory_hash: &str, published_hash: &str) -> String{
    if directory_hash == published_hash {
        println!("Hash {:?} already published. Returning", directory_hash);
        return published_hash.to_string();
    }
    println!("Publishing directory with hash: {:?}", directory_hash);
    let output = Command::new("ipfs")
                    .arg("name")
                    .arg("publish")
                    .arg(directory_hash)
                    .output()
                    .expect("ipfs name publish failed");

    let out_string = String::from_utf8_lossy(&output.stdout);
    println!("Output from ipns: {:?}", out_string);
    return directory_hash.to_string();
}

fn main() {
    // Set up command line tool
    let matches = App::new("Fossbox")
                          .version("1.0")
                          .author("Johan")
                          .about("Free box to drop stuff in")
                          .arg(Arg::with_name("username")
                               .short("u")
                               .long("username")
                               .help("Your username")
                               .value_name("your_username")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("password")
                               .short("pw")
                               .long("password")
                               .help("Your password")
                               .value_name("your_password")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("directory")
                               .short("d")
                               .long("dir")
                               .help("Your box directory")
                               .value_name("your_dir")
                               .required(true)
                               .takes_value(true))
                          .get_matches();

    let username = matches.value_of("username").unwrap_or("default_username");
    let password = matches.value_of("password").unwrap_or("default_password");
    let directory = matches.value_of("directory").unwrap_or("default_dir");

    println!("Got username {:?} and password {:?}, directory {:?}", username, password, directory);

    // Start watching the specified directory
    if let Err(err) = watch(directory) {
        println!("Error! {:?}", err)
    }
}
