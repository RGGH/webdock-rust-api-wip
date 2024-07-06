use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::collections::HashMap;
use serde_json::json;


#[derive(Debug)]
pub enum WebdockError {
    ReqwestError(reqwest::Error),
    WebdockException(String),
    ValidationException(String),
}

impl From<reqwest::Error> for WebdockError {
    fn from(err: reqwest::Error) -> Self {
        WebdockError::ReqwestError(err)
    }
}

pub struct Webdock {
    base_url: String,
    endpoints: HashMap<&'static str, &'static str>,
    expects_raw: bool,
    client: Client,
}

impl Webdock {
    pub fn new(api_token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_token)).unwrap(),
        );
        headers.insert(
            "X-Client",
            HeaderValue::from_static("webdock-rust-sdk/v1.0.0"),
        );

        let client = Client::builder()
            .default_headers(headers.clone())
            .build()
            .unwrap();

        let endpoints: HashMap<&'static str, &'static str> = [
            ("ping", "ping"),
            ("servers", "servers"),
            ("locations", "locations"),
            ("profiles", "profiles"),
            ("images", "images"),
            ("pubkeys", "account/publicKeys"),
            ("scripts", "scripts"),
            ("hooks", "hooks"),
            ("events", "events"),
        ]
        .iter()
        .cloned()
        .collect();

        Webdock {
            base_url: String::from("https://api.webdock.io/v1"),
            endpoints,
            expects_raw: false,
            client,
        }
    }

    fn send_response(&self, res: Response, json: bool) -> Result<(), WebdockError> {
        match res.status().as_u16() {
            200 | 201 | 202 | 418 => {
                if self.expects_raw {
                    // Handle raw response if expected
                    Ok(())
                } else {
                    if json {
                        let data: serde_json::Value = res.json()?;
                        println!("Response data: {:?}", data);
                    } else {
                        println!("Response status: {}", res.status());
                        // Handle non-json response
                    }
                    Ok(())
                }
            }
            status_code => Err(WebdockError::WebdockException(format!(
                "{} Error: {}",
                status_code,
                res.status().to_string()
            ))),
        }
    }

    fn make_request(
        &self,
        endpoint: &str,
        request_type: &str,
        data: Option<&serde_json::Value>,
    ) -> Result<(), WebdockError> {
        match request_type {
            "GET" => {
                let res = self
                    .client
                    .get(&format!("{}/{}", self.base_url, endpoint))
                    .send()?;
                self.send_response(res, true)?;
            }
            "POST" | "PATCH" => {
                let res = self
                    .client
                    .post(&format!("{}/{}", self.base_url, endpoint))
                    .json(data.expect("REASON"))
                    .send()?;
                self.send_response(res, true)?;
            }
            "DELETE" => {
                let res = self
                    .client
                    .delete(&format!("{}/{}", self.base_url, endpoint))
                    .send()?;
                self.send_response(res, false)?;
            }
            _ => {
                return Err(WebdockError::ValidationException(String::from(
                    "Unsupported request type",
                )));
            }
        }
        Ok(())
    }

    pub fn ping(&self) -> Result<(), WebdockError> {
        self.make_request(self.endpoints["ping"], "GET", None)
    }

    pub fn servers(&self) -> Result<(), WebdockError> {
        self.make_request(self.endpoints["servers"], "GET", None)
    }

    pub fn provision_server(&self, data: &serde_json::Value) -> Result<(), WebdockError> {
        // Validate required fields in data
        let required_fields = ["name", "slug", "locationId", "profileSlug", "imageSlug"];
        let data_obj = data.as_object().ok_or_else(|| {
            WebdockError::ValidationException(String::from("Invalid data format"))
        })?;

        for field in &required_fields {
            if !data_obj.contains_key(*field) {
                return Err(WebdockError::ValidationException(format!(
                    "Required field {} is missing.",
                    field
                )));
            }
        }
        self.make_request(self.endpoints["servers"], "POST", Some(data))
    }
    // Add other methods following similar patterns as above...
}

fn main() {
    // Initialize Webdock instance with your API token
    let webdock = Webdock::new("");

    // Example usage
    match webdock.ping() {
        Ok(_) => println!("Ping successful!"),
        Err(e) => println!("Error: {:?}", e),
    }

    match webdock.servers(){
        Ok(server) =>println!("{:?}", server),
        Err(e)=>println!("Error {:?}", e)
    }

    let data = json!({
        "name": "rust_dem",
        "slug": "rust-dem",
        "locationId": "eu",
        "profileSlug": "",
        "imageSlug": ""
    });

    match webdock.provision_server(&data) {
        Ok(()) => println!("Server provisioned successfully"),
        Err(e) => eprintln!("Error: {:?}", e),
    }

}
