extern crate percent_encoding;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::error;
use std::fs::{read_to_string, write};
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Command {
  pub desc: String,
  pub path: String,
  #[serde(default = "bool::default")]
  pub is_default: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
  pub name: String,
  pub url: String,
  #[serde(default = "HashMap::default")]
  pub cmds: HashMap<String, Command>,
}

pub type Bookmarks = HashMap<String, Bookmark>;

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

pub fn add_or_update_bookmark(
  path: &str,
  key: String,
  bookmark: Bookmark,
) -> Result<(), Box<dyn error::Error>> {
  let mut bookmarks = get_bookmarks_from_file(path).unwrap_or_default();
  let b = bookmarks.entry(key).or_insert(Bookmark::default());
  b.name = bookmark.name;
  b.url = bookmark.url;

  write(path, serde_json::to_string(&bookmarks)?)?;

  Ok(())
}

pub fn add_or_update_command(
  path: &str,
  key: String,
  cmd: String,
  command: Command,
) -> Result<(), Box<dyn error::Error>> {
  let mut bookmarks = get_bookmarks_from_file(path).unwrap_or_default();
  let bookmark_to_update = &mut bookmarks.get_mut(&key).unwrap();

  if command.is_default {
    // Override the existing default command
    for c in bookmark_to_update.cmds.iter_mut() {
      c.1.is_default = false;
    }
  }

  *bookmark_to_update
    .cmds
    .entry(cmd)
    .or_insert(Command::default()) = command;

  write(path, serde_json::to_string(&bookmarks)?)?;

  Ok(())
}

pub fn remove_bookmark(path: &str, key: String) -> Result<(), Box<dyn error::Error>> {
  let mut bookmarks = get_bookmarks_from_file(path).unwrap_or_default();
  bookmarks.remove(&key);

  write(path, serde_json::to_string(&bookmarks)?)?;

  Ok(())
}

pub fn remove_command(path: &str, key: String, cmd: String) -> Result<(), Box<dyn error::Error>> {
  let mut bookmarks = get_bookmarks_from_file(path).unwrap_or_default();
  let commands = &mut bookmarks.get_mut(&key).unwrap().cmds;
  commands.remove(&cmd);

  write(path, serde_json::to_string(&bookmarks)?)?;

  Ok(())
}

fn construct_url(base_url: &str, path: &str, query: &str) -> String {
  let encoded_query = utf8_percent_encode(&query, FRAGMENT).to_string();
  let full_url = format!("{}{}", base_url, path);
  let url = &full_url.replace("{}", &encoded_query);

  url.to_string()
}

pub fn construct_bookmark_url(bookmark: &mut Bookmark, (command, query): (&str, &str)) -> String {
  if command.is_empty() || bookmark.cmds.is_empty() {
    return bookmark.url.to_string();
  }

  match bookmark.cmds.entry(command.to_string()) {
    Occupied(cmd) => construct_url(&bookmark.url, &cmd.get().path, query),
    Vacant(_) => construct_url(
      &bookmark.url,
      match &bookmark.cmds.values().find(|x| x.is_default) {
        Some(cmd) => &cmd.path,
        None => "",
      },
      &vec![command, query]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>()
        .join(" "),
    ),
  }
}
