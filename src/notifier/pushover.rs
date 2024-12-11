use anyhow::Result;

pub struct PushoverNotifier {
    token: String,
    user_key: String,
    client: reqwest::Client,
}

impl PushoverNotifier {
    pub fn new(token: String, user_key: String) -> Self {
        Self {
            token,
            user_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        let params = [
            ("token", self.token.as_str()),
            ("user", self.user_key.as_str()),
            ("message", message),
        ];

        self.client
            .post("https://api.pushover.net/1/messages.json")
            .form(&params)
            .send()
            .await?;

        Ok(())
    }
}