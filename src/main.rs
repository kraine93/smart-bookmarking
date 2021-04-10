#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::Redirect;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::time::Instant;
mod utils;

#[get("/")]
fn index() -> &'static str {
    "Hello World!"
}

#[get("/search?<cmd>")]
fn search(cmd: String) -> Redirect {
    let start = Instant::now();

    let mut bookmarks =
        utils::bookmarks::get_bookmarks_from_file("src/bookmarks.json").unwrap_or_default();

    let (command, query) = utils::get_command_from_query_string(&cmd);

    let redirect_url = match bookmarks.entry(command.to_string()) {
        Occupied(mut bookmark) => utils::bookmarks::construct_bookmark_url(
            bookmark.get_mut(),
            utils::get_command_from_query_string(query),
        ),
        Vacant(_) => utils::google::construct_google_search_query(&cmd),
    };

    println!("Time elapsed: {:?}", start.elapsed());

    Redirect::to(redirect_url)
}

fn main() {
    rocket::ignite().mount("/", routes![index, search]).launch();
}
