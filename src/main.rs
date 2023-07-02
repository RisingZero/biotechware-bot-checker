pub mod biotechware;
pub mod telegram;

use biotechware::{BiotechwareApi, ListType, RecordType};
use chrono::{Utc, Datelike, Month};
use std::collections::HashMap;
use std::env;

async fn resumes_main() {
    let mut biotech = BiotechwareApi::new(
        env::var("BTW_USERNAME").unwrap(),
        env::var("BTW_PASSWORD").unwrap(),
    );
    let telegram = telegram::TelegramBot::new(
        env::var("TG_TOKEN_RESUMES").unwrap(),
        env::var("TG_CHAT_ID_RESUMES").unwrap(),
    );

    loop {
        let mut stats: HashMap<RecordType,f64> = HashMap::new();

        let records = biotech.get_records_of_month(
            ListType::Reported, 
            Month::try_from(u8::try_from(Utc::now().month()).unwrap()).unwrap()
        ).await.unwrap();

        records.into_iter().for_each(|r| {
            match stats.get_mut(&r.record_type_id) {
                Some(v) => {
                    *v += r.record_type_id.price_value();
                },
                None => {
                    stats.insert(r.record_type_id, r.record_type_id.price_value());
                }
            }
        });

        let total_taxed = stats.values().sum::<f64>() * (1.0 - biotechware::TAX_RATE);
        let mut message = String::new();
        stats.into_iter().for_each(|(k,v)| {
            message.push_str(format!("{}: {:.2}€\n", k.to_str(), v).as_str());
        });
        message.push_str(format!("Totale (tassato): {:.2}€", total_taxed).as_str());
        telegram.send_message(message).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        //tokio::time::sleep(tokio::time::Duration::from_secs(3600*3)).await;
        break; // TO REMOVE
    }

}

async fn checker_main() {
    let mut biotech = BiotechwareApi::new(
        env::var("BTW_USERNAME").unwrap(),
        env::var("BTW_PASSWORD").unwrap(),
    );
    
    loop {
        let records = biotech.get_all_records(ListType::Unreported).await.unwrap();

        println!("{:?}", records);
        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        break;
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let rh = tokio::spawn(
        async move {
            resumes_main().await
        }
    );

    let ch = tokio::spawn(
        async move {
            checker_main().await
        }
    );

    //rh.await?;
    ch.await?;

    Ok(())
}
