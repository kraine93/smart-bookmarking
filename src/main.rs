#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::{RawStr, Status};
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
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
        Vacant(_) => match cmd.as_str() {
            "/help" => String::from("/bookmarks"),
            _ => utils::google::construct_google_search_query(&cmd),
        },
    };

    println!("Time elapsed: {:?}", start.elapsed());

    Redirect::to(redirect_url)
}

#[derive(serde::Serialize)]
struct BookmarksContext {
    query: String,
    bookmarks: Bookmarks,
}

#[derive(serde::Serialize)]
struct BookmarkContext<'a> {
    shortcut: String,
    bookmark: &'a Bookmark,
}

#[derive(serde::Serialize)]
struct CommandContext<'a> {
    shortcut: String,
    cmd: String,
    command: &'a Command,
}

#[get("/bookmarks?<query>")]
pub fn get_bookmarks(query: Option<&RawStr>) -> Template {
    let mut bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    if let Some(q) = query {
        println!("{}", q);
        bookmarks = bookmarks
            .into_iter()
            .filter(|(_k, v)| {
                v.name
                    .to_lowercase()
                    .contains(&q.to_string().to_lowercase())
            })
            .collect();
    }

    Template::render(
        "bookmarks",
        BookmarksContext {
            query: match query {
                Some(q) => q.to_string(),
                None => String::new(),
            },
            bookmarks: bookmarks,
        },
    )
}

#[get("/bookmarks/add")]
pub fn add_bookmark_template() -> Template {
    Template::render(
        "add-bookmark",
        BookmarkContext {
            shortcut: String::new(),
            bookmark: &Bookmark::default(),
        },
    )
}

#[get("/bookmarks/<key>")]
pub fn get_bookmark(key: &RawStr) -> Template {
    let bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    let bookmark = bookmarks.get(&key.to_lowercase().to_string()).expect("");

    Template::render(
        "bookmark",
        BookmarkContext {
            shortcut: key.to_lowercase().to_string(),
            bookmark: bookmark,
        },
    )
}

#[get("/bookmarks/<key>/edit")]
pub fn edit_bookmark(key: &RawStr) -> Template {
    let bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    let bookmark = bookmarks.get(&key.to_lowercase().to_string()).expect("");

    Template::render(
        "add-bookmark",
        BookmarkContext {
            shortcut: key.to_lowercase().to_string(),
            bookmark: bookmark,
        },
    )
}

#[get("/bookmarks/<key>/add")]
pub fn add_command_template(key: &RawStr) -> Template {
    let bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    bookmarks.get(&key.to_lowercase().to_string()).expect("");

    Template::render(
        "add-command",
        CommandContext {
            shortcut: key.to_lowercase().to_string(),
            cmd: String::new(),
            command: &Command::default(),
        },
    )
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

    let bookmarks =
        utils::bookmarks::get_bookmarks_from_file(BOOKMARKS_FILE_PATH).unwrap_or_default();

    if let Some(_existing_bookmark) = bookmarks.get(&key.to_lowercase().to_string()) {
        return ApiResponse::new()
            .set_status(Status::Conflict)
            .set_message("A bookmark with this shortcut already exists!");
    }

    match utils::bookmarks::add_or_update_bookmark(
        BOOKMARKS_FILE_PATH,
        key.to_lowercase().to_string(),
        bookmark.into_inner(),
    ) {
        Ok(_) => return ApiResponse::new().set_message("Bookmark added!"),
        Err(_) => {
            return ApiResponse::new()
                .set_status(Status::InternalServerError)
                .set_message("Something went wrong! Please try again.")
        }
    }
}

#[patch("/bookmarks/<key>", data = "<bookmark>")]
fn update_bookmark(key: &RawStr, bookmark: Json<Bookmark>) -> ApiResponse {
    match utils::bookmarks::add_or_update_bookmark(
        BOOKMARKS_FILE_PATH,
        key.to_lowercase().to_string(),
        bookmark.into_inner(),
    ) {
        Ok(_) => return ApiResponse::new().set_message("Bookmark updated!"),
        Err(_) => {
            return ApiResponse::new()
                .set_status(Status::InternalServerError)
                .set_message("Something went wrong! Please try again.")
        }
    }
}

#[post("/bookmarks/<key>/commands/<cmd>", data = "<command>")]
fn add_command(key: &RawStr, cmd: &RawStr, command: Json<Command>) -> ApiResponse {
    match utils::bookmarks::add_or_update_command(
        BOOKMARKS_FILE_PATH,
        key.to_lowercase().to_string(),
        cmd.to_lowercase().to_string(),
        command.into_inner(),
    ) {
        Ok(_) => {
            return ApiResponse::new()
                .set_status(Status::Ok)
                .set_message("Command added!")
        }
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[delete("/bookmarks/<key>")]
fn remove_bookmark(key: &RawStr) -> ApiResponse {
    match utils::bookmarks::remove_bookmark(BOOKMARKS_FILE_PATH, key.to_lowercase().to_string()) {
        Ok(_) => {
            return ApiResponse::new()
                .set_status(Status::Ok)
                .set_message("Bookmark deleted!")
        }
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[delete("/bookmarks/<key>/commands/<cmd>")]
fn remove_command(key: &RawStr, cmd: &RawStr) -> ApiResponse {
    match utils::bookmarks::remove_command(
        BOOKMARKS_FILE_PATH,
        key.to_lowercase().to_string(),
        cmd.to_lowercase().to_string(),
    ) {
        Ok(_) => return ApiResponse::new().set_status(Status::Ok),
        Err(_) => return ApiResponse::new().set_status(Status::InternalServerError),
    }
}

#[catch(404)]
fn not_found() -> String {
    format!("That page doesn't exist. Try something else?")
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                search,
                get_bookmarks,
                get_bookmark,
                add_bookmark,
                edit_bookmark,
                update_bookmark,
                add_bookmark_template,
                add_command,
                add_command_template,
                remove_bookmark,
                remove_command
            ],
        )
        .mount("/static", StaticFiles::from("static"))
        .attach(Template::fairing())
        .register(catchers![not_found])
        .launch();
}
