#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::{RawStr, Status};
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::time::Instant;
mod utils;
use utils::bookmarks::{Bookmark, Bookmarks, Command};
use utils::ApiResponse;

const BOOKMARKS_FILE_PATH: &'static str = "src/bookmarks.json";

#[get("/")]
fn index() -> &'static str {
    "Hello World!"
}

#[get("/search?<cmd>")]
fn search(cmd: String) -> Redirect {
    let start = Instant::now();

    let mut bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

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

#[get("/bookmarks")]
pub fn get_bookmarks() -> Json<Bookmarks> {
    let bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    Json(bookmarks)
}

#[post("/bookmarks/<key>", data = "<bookmark>")]
fn add_bookmark(key: &RawStr, bookmark: Json<Bookmark>) -> ApiResponse {
    if bookmark
        .cmds
        .values()
        .into_iter()
        .filter(|x| x.is_default)
        .collect::<Vec<_>>()
        .len()
        > 1
    {
        return ApiResponse::new()
            .set_status(Status::BadRequest)
            .set_message("Only one command can be set as default!");
    }

    match utils::bookmarks::add_or_update_bookmark(
        BOOKMARKS_FILE_PATH,
        key.to_string(),
        bookmark.into_inner(),
    ) {
        Ok(_) => return ApiResponse::new(),
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[post("/bookmarks/<key>/commands/<cmd>", data = "<command>")]
fn add_command(key: &RawStr, cmd: &RawStr, command: Json<Command>) -> ApiResponse {
    match utils::bookmarks::add_or_update_command(
        BOOKMARKS_FILE_PATH,
        key.to_string(),
        cmd.to_string(),
        command.into_inner(),
    ) {
        Ok(_) => return ApiResponse::new().set_status(Status::Ok),
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[delete("/bookmarks/<key>")]
fn remove_bookmark(key: &RawStr) -> ApiResponse {
    match utils::bookmarks::remove_bookmark(BOOKMARKS_FILE_PATH, key.to_string()) {
        Ok(_) => return ApiResponse::new().set_status(Status::Ok),
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[delete("/bookmarks/<key>/commands/<cmd>")]
fn remove_command(key: &RawStr, cmd: &RawStr) -> ApiResponse {
    match utils::bookmarks::remove_command(BOOKMARKS_FILE_PATH, key.to_string(), cmd.to_string()) {
        Ok(_) => return ApiResponse::new().set_status(Status::Ok),
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                search,
                get_bookmarks,
                add_bookmark,
                add_command,
                remove_bookmark,
                remove_command
            ],
        )
        .launch();
}
