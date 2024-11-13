extern crate core;

use reqwest;
use reqwest::blocking::Client;
use soup::{NodeExt, QueryBuilderExt};

pub use crate::reqwest::Error;

const CAS_LOGIN_URL: &str = "https://cas.univ-lyon1.fr/cas/login";
const CAS_LOGOUT_URL: &str = "https://cas.univ-lyon1.fr/cas/logout";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.3";

pub struct Lyon1CasClient {
    reqwest_client: Client,
    authenticated: bool,
}

impl Lyon1CasClient {
    pub fn new() -> Self { Self { reqwest_client: Client::builder().user_agent(USER_AGENT).cookie_store(true).build().unwrap(), authenticated: false } }

    pub fn authenticated(&self) -> bool { self.authenticated }

    pub fn authenticate_user(&mut self, username: &str, password: &str) -> Result<bool, Error> {
        let response = self.reqwest_client.post(CAS_LOGIN_URL).form(
            &[
                ("username", username),
                ("password", password),
                ("execution", &self.get_exec_token().unwrap()),
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

    pub fn logout(&mut self) -> Result<bool, Error> {
        self.authenticated = false;
        self.reqwest_client.get(CAS_LOGOUT_URL).send().map(|response| response.status().is_success())
    }

    pub fn service_request(&self, service: impl Into<String>, unsafe_req: bool) -> Result<String, Error> {
        let mut service = service.into();
        if unsafe_req { service += "/unsafe=1" }

        self.reqwest_client.get(CAS_LOGIN_URL)
            .query::<[(String, String); 1]>(&[("service".to_owned(), service.into())])
            .send()
            .map(|response| response.text())?
    }

    fn get_exec_token(&self) -> Result<String, Error> {
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
        println!("Exec Token: {}", token);

        assert!(token.len() > 0);
    }

    #[test]
    fn authenticate_user() {
        let dotenv = DotEnv::new("");
        let (username, password) = (dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());

        let mut cas_client = Lyon1CasClient::new();
        assert!(cas_client.authenticate_user(&username, &password).unwrap());
    }

    #[test]
    fn logout() {
        let dotenv = DotEnv::new("");
        let (username, password) = (dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());

        let mut cas_client = Lyon1CasClient::new();
        assert!(!cas_client.authenticated());

        assert!(cas_client.authenticate_user(&username, &password).unwrap());
        assert!(cas_client.authenticated());

        assert!(cas_client.logout().unwrap());
    }

    #[test]
    fn service_request() {
        let dotenv = DotEnv::new("");
        let (username, password) = (dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());
        let mut cas_client = Lyon1CasClient::new();

        assert!(cas_client.authenticate_user(&username, &password).unwrap());

        println!("{}", cas_client.service_request("https://tomuss.univ-lyon1.fr", true).unwrap());
    }
}
