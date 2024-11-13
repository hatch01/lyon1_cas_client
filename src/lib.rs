extern crate core;

use reqwest::blocking::Client;
use soup::{NodeExt, QueryBuilderExt};

const CAS_LOGIN_URL: &str = "https://cas.univ-lyon1.fr/cas/login";
//const cas_logout: &str = "https://cas.univ-lyon1.fr/cas/logout";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.3";

struct Lyon1CasClient {
    reqwest_client: Client,
    authenticated: bool,
}

struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    fn new(username: String, password: String) -> Credentials {
        Self { username, password }
    }
}

impl Lyon1CasClient {
    pub fn new() -> Self { Self { reqwest_client: Client::builder().user_agent(USER_AGENT).cookie_store(true).build().unwrap(), authenticated: false } }

    pub fn authenticated(&self) -> bool { self.authenticated }

    pub fn authenticate_user(&mut self, credentials: Credentials) -> Result<bool, reqwest::Error> {
        let response = self.reqwest_client.post(CAS_LOGIN_URL).form(
            &[
                ("username", credentials.username.as_str()),
                ("password", credentials.password.as_str()),
                ("execution", self.get_exec_token().unwrap().as_str()),
                ("_eventId", "submit"),
            ]
        ).send()?;

        if !response.status().is_success() {
            println!("Failed to login to CAS (Status: {})", response.status());

            return Ok(false);
        }

        self.authenticated = true;
        Ok(true)
    }

    pub fn logout(&mut self) -> Result<bool, reqwest::Error> {
        self.authenticated = false;
        self.reqwest_client.get(CAS_LOGIN_URL).send().map(|response| response.status().is_success())
    }

    fn get_exec_token(&self) -> Result<String, reqwest::Error> {
        let response = self.reqwest_client.get(CAS_LOGIN_URL).send()?;

        let soup = soup::Soup::new(&response.text()?);
        let token = soup.attr("name", "execution").find().unwrap().get("value").unwrap();

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dotenv::dotenv::DotEnv;

    #[test]
    fn exec_token() {
        let token = Lyon1CasClient::new().get_exec_token().unwrap();
        println!("{}", token);

        assert!(token.len() > 0);
    }

    #[test]
    fn authenticate_user() {
        let dotenv = DotEnv::new("");
        let credentials = Credentials::new(dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());

        let mut cas_client = Lyon1CasClient::new();
        assert!(cas_client.authenticate_user(credentials).unwrap());
    }

    #[test]
    fn logout() {
        let dotenv = DotEnv::new("");
        let credentials = Credentials::new(dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());

        let mut cas_client = Lyon1CasClient::new();
        assert!(!cas_client.authenticated());
        
        assert!(cas_client.authenticate_user(credentials).unwrap());
        assert!(cas_client.authenticated());
         
        assert!(cas_client.logout().unwrap());
    }
}
