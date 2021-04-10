use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{Request, Response};

pub mod bookmarks;
pub mod google;

pub fn get_command_from_query_string(query: &str) -> (&str, &str) {
    if query.contains(' ') {
        let index_of_space = query.find(' ').unwrap_or(0);
        return (&query[..index_of_space], &query[index_of_space + 1..]);
    }

    (query, "")
}

#[derive(Debug)]
pub struct ApiResponse {
    status: Status,
    message: String,
}

impl ApiResponse {
    pub fn new() -> ApiResponse {
        ApiResponse {
            status: Status::Ok,
            message: String::new(),
        }
    }

    pub fn set_status(self, status: Status) -> ApiResponse {
        ApiResponse {
            status: status,
            ..self
        }
    }

    pub fn set_message(self, message: &str) -> ApiResponse {
        ApiResponse {
            message: String::from(message),
            ..self
        }
    }
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        Response::build_from(self.message.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_command_from_query_string_no_whitespace() {
        let actual = get_command_from_query_string("tw");
        let expected = "tw";
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_command_from_query_string_with_whitespace() {
        let actual = get_command_from_query_string("tw @fbOpenSource");
        let expected = "tw";
        assert_eq!(expected, actual);
    }
}
