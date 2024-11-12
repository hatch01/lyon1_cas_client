use soup::{NodeExt, QueryBuilderExt};
use reqwest::blocking::Client;

const CAS_LOGIN_URL: &str = "https://cas.univ-lyon1.fr/cas/login";
//const cas_logout: &str = "https://cas.univ-lyon1.fr/cas/logout";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.3";

struct Lyon1CasClient {
    authenticated: bool,
    reqwest_client: Client,
}

struct Credentials {
    username: String,
    password: String,
    tgc_token: Option<String>,
}

impl Credentials {
    pub fn new(username: String, password: String) -> Credentials {
        Self {
            username,
            password,
            tgc_token: None,
        }
    }
}

impl Lyon1CasClient {
    pub fn new() -> Self { Self { authenticated: false, reqwest_client: reqwest::blocking::Client::builder().user_agent(USER_AGENT).build().unwrap() } }

    pub fn authenticated(&self) -> bool { self.authenticated }

    pub fn authenticate_user(&self, credentials: &mut Credentials) -> bool {
        let response = self.reqwest_client.post(CAS_LOGIN_URL).form(
            &[
                ("username", credentials.username.as_str()),
                ("password", credentials.password.as_str()),
                ("execution", self.get_exec_token().unwrap().as_str()),
                ("_eventId", "submit"),
            ]
        ).send().unwrap();

        if !response.status().is_success() {
            println!("{}", response.status());
            println!("{}", response.text().unwrap());

            return false;
        }

        let token = response.cookies().filter(|cookie| cookie.name() == "TGC-CAS").next().unwrap().value().to_owned();
        credentials.tgc_token = Some(token);
        true
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
    use rust_dotenv::dotenv::DotEnv;
    use super::*;

    #[test]
    fn exec_token() {
        let token = Lyon1CasClient::new().get_exec_token().unwrap();
        println!("{}", token);

        assert!(token.len() > 0);
    }

    #[test]
    fn authenticate_user() {
        let dotenv = DotEnv::new("");
        
        let mut credentials = Credentials::new(dotenv.get_var("USERNAME".to_string()).unwrap(), dotenv.get_var("PASSWORD".to_string()).unwrap());
        let success = Lyon1CasClient::new().authenticate_user(&mut credentials);
        println!("success: {:?}", success);
        assert!(success);

        if success {
            println!("token: {}", credentials.tgc_token.unwrap());
        }
    }
}
