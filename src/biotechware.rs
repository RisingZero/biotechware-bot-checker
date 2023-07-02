pub mod models;

use core::panic;
use chrono::{DateTime, Utc, Datelike};
use reqwest::StatusCode;
use futures::{stream, StreamExt};
pub use models::{LoginRequest, GetRecordsRequest, GetRecordsResponse, Record, ListType, RecordType};

const API_URL: &str = "https://portal.biotechware.com";
const DEFAULT_TYPE_LOG: &str = "physician";
const DEFAULT_GET_RECORDS_COUNT: i32 = 11;
const MAX_PARALLEL_REQUESTS: usize = 10;
pub const TAX_RATE: f64 = 0.2;

pub struct BiotechwareApi {
    last_logged: Option<DateTime<Utc>>,
    username: String,
    password: String,
    client: reqwest::Client
}

impl BiotechwareApi {

    pub fn new(username: String, password: String) -> BiotechwareApi {
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .build().unwrap();

        BiotechwareApi {
            username,
            password,
            last_logged: None,
            client
        }
    }

    pub async fn login(&mut self) -> Result<(), reqwest::Error> {
        let login_result = self.client.post(
            format!("{}/login_handler?__logins=0&came_from=%2F", API_URL))
            .form(&LoginRequest {
                login: self.username.clone(),
                password: self.password.clone(),
            })
            .send().await?;

        match login_result.status() { 
            StatusCode::OK => {
                self.last_logged = Some(Utc::now());
                Ok(())
            }
            _ => {
                panic!("Login failed with status: {}", login_result.status());
            }
        }
    }

    pub async fn get_records(&mut self, list_type: ListType, page: i32, num_of_pages: i32) -> Result<Vec<Record>, reqwest::Error> {
        match self.last_logged {
            Some(last_logged) => {
                if Utc::now() - last_logged > chrono::Duration::hours(1) {
                    self.login().await.expect("Failed to login");
                }
            },
            None => {
                self.login().await.expect("Failed to login");
            }
        }

        let mut counters = vec![(page - 1) * DEFAULT_GET_RECORDS_COUNT + 1];
        while counters.len() < num_of_pages as usize {
            counters.push(counters.last().unwrap() + DEFAULT_GET_RECORDS_COUNT);
        }

        let mut results = stream::iter(counters)
            .map(|c| {
                let client = self.client.clone();
                tokio::spawn(async move {
                    let resp = client.get(
                        format!("{}/manage/ecg/get_other_records", API_URL))
                        .query(&GetRecordsRequest {
                            counter: c,
                            type_log: String::from(DEFAULT_TYPE_LOG),
                            list_type,
                            record_types_filter:  "".to_owned(),
                            search_filter: "".to_owned(),
                        })
                        .send().await?;

                    Ok::<Vec<Record>,reqwest::Error>(resp.json::<GetRecordsResponse>().await?.list)
                })
            }).buffered(MAX_PARALLEL_REQUESTS);

        let mut records: Vec<Record> = Vec::with_capacity((num_of_pages * DEFAULT_GET_RECORDS_COUNT) as usize);
        loop { 
            let ro = results.next().await;
            match ro {
                Some(r) => {
                    records.extend_from_slice(r.unwrap().unwrap().as_mut_slice());
                },
                None => {
                    break;
                }
            }
        }

        Ok(records)
    }

    pub async fn get_records_of_month(&mut self, list_type: ListType, month: chrono::Month) -> Result<Vec<Record>, reqwest::Error> {
        let mut records: Vec<Record> = vec![];
        let mut page_counter = 0;
        loop {
            let mut results = self.get_records(list_type, page_counter, MAX_PARALLEL_REQUESTS as i32).await?;
            results = results.into_iter()
                .filter(|rec| rec.last_report_date.unwrap().month() == month.number_from_month())
                .collect::<Vec<Record>>();
            match results.len() {
                0 => break,
                _ => {
                    records.extend(results);
                    page_counter += MAX_PARALLEL_REQUESTS as i32;
                }
            }
        }

        Ok(records)
    }

    pub async fn get_all_records(&mut self, list_type: ListType) -> Result<Vec<Record>, reqwest::Error> {
        let mut records: Vec<Record> = vec![];
        let mut page_counter = 0;
        loop {
            let results = self.get_records(list_type, page_counter, MAX_PARALLEL_REQUESTS as i32).await?;

            if results.len() < MAX_PARALLEL_REQUESTS * DEFAULT_GET_RECORDS_COUNT as usize {
                records.extend(results);
                break;
            }
            records.extend(results);
            page_counter += MAX_PARALLEL_REQUESTS as i32;
        }

        Ok(records)
    }
}