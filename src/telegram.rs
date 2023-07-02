use self::models::SendMessageBody;

pub mod models;

pub struct TelegramBot {
    token: String,
    chat_id: String,
}

impl TelegramBot {

    pub fn new(token: String, chat_id: String) -> Self {
        TelegramBot {
            token,
            chat_id,
        }
    }

    pub async fn send_message(&self, message: String) {
        let client = reqwest::Client::new();

        client.post(
            format!("https://api.telegram.org/bot{}/sendMessage", self.token)
        )
        .json(&SendMessageBody {
            chat_id: self.chat_id.clone(),
            text: message,
            disable_web_page_preview: true,
        })
        .send().await.unwrap();
    }
}