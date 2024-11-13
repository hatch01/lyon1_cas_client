use reqwest::blocking::Client;
use soup::{NodeExt, QueryBuilderExt};

const CAS_LOGIN_URL: &str = "https://cas.univ-lyon1.fr/cas/login";
//const cas_logout: &str = "https://cas.univ-lyon1.fr/cas/logout";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.3";

struct Lyon1CasClient {
    reqwest_client: Client,
}

enum Credentials {
    Unauthenticated { username: String, password: String },
    Authenticated { username: String, password: String, token: String },
}

impl Lyon1CasClient {
    pub fn new() -> Self { Self { reqwest_client: Client::builder().user_agent(USER_AGENT).cookie_store(true).build().unwrap() } }

    pub fn authenticate_user(&self, credentials: Credentials) -> Option<Credentials> {
        if let Credentials::Unauthenticated { username, password } = credentials {
            let response = self.reqwest_client.post(CAS_LOGIN_URL).form(
                &[
                    ("username", username.as_str()),
                    ("password", password.as_str()),
                    ("execution", self.get_exec_token().unwrap().as_str()),
                    ("_eventId", "submit"),
                ]
            ).send().unwrap();

            if !response.status().is_success() {
                println!("Failed to login to CAS (Status: {})", response.status());

                return None;
            }

            let token = response.cookies().filter(|cookie| cookie.name() == "TGC-CAS").next().unwrap().value().to_owned();
            Some(Credentials::Authenticated { username, password, token })
        } else {
            Some(credentials)
        }
    }
    
    pub fn logout(&self) -> Result<(), reqwest::Error>{
        self.reqwest_client.get(CAS_LOGIN_URL).send().map(|_| ())
    }

    pub(crate) fn get_exec_token(&self) -> Result<String, reqwest::Error> {
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

        let mut credentials = Credentials::Unauthenticated {
            username: dotenv.get_var("USERNAME".to_string()).unwrap(),
            password: dotenv.get_var("PASSWORD".to_string()).unwrap(),
        };

        let credentials_opt = Lyon1CasClient::new().authenticate_user(credentials);
        assert!(credentials_opt.is_some());

        if let Some(Credentials::Authenticated { username, password, token }) = credentials_opt {
            println!("Username: {}", username);
            println!("Token: {}", token);
        }
    }
}
