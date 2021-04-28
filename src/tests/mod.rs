mod scenario;

#[cfg(test)]
mod test {   
    
    use crate::routing::{*, user::Login};
    use dotenv::dotenv;
    use rocket::{http::{ContentType, Status}, local::Client};
    use super::scenario::{ClientActions, test_user};

    #[test]
    fn simple () {
        dotenv().ok();

        let user = test_user ();

        let server = start();
        let client = Client::new(server).unwrap();
        client.register_user (&user).unwrap();
        client.login (&Login {
            username: user.username,
            password: user.password,
        }).unwrap();
    }

    #[test]
    fn clear () {
        dotenv().ok();

        let server = start();
        assert_eq!(Client::new(server).unwrap()
                       .post("/login")
                       .body(r#"{"username":"wrong"}"#)
                       .header(ContentType::JSON)
                       .dispatch()
                       .status(), Status::UnprocessableEntity)
    }
}
