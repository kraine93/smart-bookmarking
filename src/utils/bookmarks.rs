extern crate percent_encoding;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::error;
use std::fs::read_to_string;
use std::io;
use std::path::Path;

pub type Bookmarks = HashMap<String, Bookmark>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Command {
  path: String,
  #[serde(default = "bool::default")]
  is_default: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
  pub url: String,
  pub cmds: HashMap<String, Command>,
}

fn read_bookmarks_file(path: &str) -> Result<String, io::Error> {
  let file_string = read_to_string(Path::new(path))?;
  Ok(file_string)
}

fn parse_bookmarks_json(file_string: &str) -> Result<Bookmarks, serde_json::Error> {
  let bookmarks: Bookmarks = serde_json::from_str(file_string)?;
  Ok(bookmarks)
}

pub fn get_bookmarks_from_file(path: &str) -> Result<Bookmarks, Box<dyn error::Error>> {
  let file_string = read_bookmarks_file(path)?;
  let bookmarks = parse_bookmarks_json(&file_string)?;
  Ok(bookmarks)
}

fn construct_url(base_url: &str, path: &str, query: &str) -> String {
  let encoded_query = utf8_percent_encode(&query, FRAGMENT).to_string();
  let full_url = format!("{}{}", base_url, path);
  let url = &full_url.replace("{}", &encoded_query);

  url.to_string()
}

pub fn construct_bookmark_url(bookmark: &mut Bookmark, (command, query): (&str, &str)) -> String {
  if command.is_empty() || bookmark.cmds.len() == 0 {
    return bookmark.url.to_string();
  }

  match bookmark.cmds.entry(command.to_string()) {
    Occupied(cmd) => construct_url(&bookmark.url, &cmd.get().path, query),
    Vacant(_) => construct_url(
      &bookmark.url,
      &bookmark.cmds.values().find(|x| x.is_default).unwrap().path,
      &vec![command, query]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>()
        .join(" "),
    ),
  }
}
