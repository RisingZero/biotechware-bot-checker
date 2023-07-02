use serde::{Serialize, Deserialize, Serializer, Deserializer};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRecordsRequest {
    pub counter: i32,
    pub type_log: String,
    pub list_type: ListType,
    pub record_types_filter: String,
    #[serde(rename = "searchFilter")]
    pub search_filter: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRecordsResponse {
    pub count: i32,
    pub counter: i32,
    pub list: Vec<Record>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub status: String,
    pub id: String,
    #[serde(rename = "timezone_indication")]
    pub timezone_indication: String,
    pub last_report_revision: String,
    #[serde(with = "report_date_format")]
    pub last_report_date: Option<DateTime<Utc>>,
    pub record_type_id: RecordType,
    #[serde(rename = "record_type_name")]
    pub record_type_name: String,
    pub firstname: String,
    pub lastname: String,
    pub url: String,
    pub health_code: Option<String>,
    pub reception_date: String,
    pub date: String,
    pub requester: String,
    pub effective_service_level: Option<ServiceLevel>,
}

/*
    List Type
    - reportedRecords
    - unreportedRecords
 */
#[derive(Debug, Clone, Copy)]
pub enum ListType {
    Reported,
    Unreported,
}

impl ListType {
    pub fn to_str(&self) -> &'static str {
        match self {
            ListType::Reported => "reportedRecords",
            ListType::Unreported => "unreportedRecords",
        }
    }

    pub fn from_str(s: &str) -> Result<ListType, Box<dyn std::error::Error>> {
        match s {
            "reportedRecords" => Ok(ListType::Reported),
            "unreportedRecords" => Ok(ListType::Unreported),
            _ => panic!("Unknown ListType: {}", s),
        }
    }
}

impl Serialize for ListType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
    where S: Serializer {
        serializer.serialize_str(self.to_str())
    }
}

impl<'de> Deserialize<'de> for ListType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(ListType::from_str(s.as_str()).unwrap())
    }
}

/*
    Record Type
    - 1         ECG a riposo
    - 3         Holter ECG
    - 5         ABPM
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordType {
    Ecg = 1,
    HolterEcg = 3,
    Abpm = 5,
}

impl RecordType {
    pub fn to_str(&self) -> &'static str {
        match self {
            RecordType::Ecg => "ECG a riposo",
            RecordType::HolterEcg => "Holter ECG",
            RecordType::Abpm => "ABPM",
        }
    }

    pub fn from_str(s: &str) -> Result<RecordType, Box<dyn std::error::Error>> {
        match s {
            "ECG a riposo" => Ok(RecordType::Ecg),
            "Holter ECG" => Ok(RecordType::HolterEcg),
            "ABPM" => Ok(RecordType::Abpm),
            _ => panic!("Unknown RecordType: {}", s),
        }
    }

    pub fn price_value(&self) -> f64 {
        match self {
            RecordType::Ecg => 4.0,
            RecordType::HolterEcg => 15.0,
            RecordType::Abpm => 8.0,
        }
    }
}

impl Serialize for RecordType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
    where S: Serializer {
        match self {
            RecordType::Ecg => serializer.serialize_i32(1),
            RecordType::HolterEcg => serializer.serialize_i32(3),
            RecordType::Abpm => serializer.serialize_i32(5),
        }
    }
}

impl<'de> Deserialize<'de> for RecordType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = i32::deserialize(deserializer)?;
        match s {
            1 => Ok(RecordType::Ecg),
            3 => Ok(RecordType::HolterEcg),
            5 => Ok(RecordType::Abpm),
            _ => panic!("Unknown RecordType: {}", s),
        }
    }
}

/*
    Last Report Date serializer/deserializer
 */
mod report_date_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%d/%m/%Y %H:%M";

    pub fn serialize<S>(
        date: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => {
                let s = format!("{}", d.format(FORMAT));
                serializer.serialize_str(&s)        
            },
            None => serializer.serialize_str("NA")
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "NA" => Ok(None),
            _ => Ok(Some(Utc.datetime_from_str(&s, FORMAT).unwrap()))
        }
    }
}

/*
    Service Level
    - TR-ECG-90
    - TR-ECG-DAY
    - TR-HC-24
    - TR-HC-48
    - TR-HP-24
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceLevel {
    ECG90,
    ECGDAY,
    HC24,
    HC48,
    HP24,
    Unknown
}

impl ServiceLevel {

    pub fn to_str(&self) -> &'static str {
        match self {
            ServiceLevel::ECG90 => "TR-ECG-90",
            ServiceLevel::ECGDAY => "TR-ECG-DAY",
            ServiceLevel::HC24 => "TR-HC-24",
            ServiceLevel::HC48 => "TR-HC-48",
            ServiceLevel::HP24 => "TR-HP-24",
            ServiceLevel::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> ServiceLevel {
        match s {
            "TR-ECG-90" => ServiceLevel::ECG90,
            "TR-ECG-DAY" => ServiceLevel::ECGDAY,
            "TR-HC-24" => ServiceLevel::HC24,
            "TR-HC-48" => ServiceLevel::HC48,
            "TR-HP-24" => ServiceLevel::HP24,
            _ => ServiceLevel::Unknown,
        }
    }

}

impl Serialize for ServiceLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
    where S: Serializer {
        serializer.serialize_str(self.to_str())
    }
}

impl<'de> Deserialize<'de> for ServiceLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(ServiceLevel::from_str(s.as_str()))
    }
}
