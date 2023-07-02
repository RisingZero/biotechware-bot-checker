use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMessageBody {
    pub chat_id: String,
    pub text: String,
    pub disable_web_page_preview: bool
}