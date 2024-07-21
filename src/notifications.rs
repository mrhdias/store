//
// Notifications
//
// References:
// https://www.youtube.com/watch?v=VhQxKGzkQ1c
// https://www.courier.com/guides/rust-send-email/
//

use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::message::{SinglePart, header::ContentType};
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;

pub struct SMTP {
    username: String,
    password: String,
    server: String,
}

impl SMTP {
    pub async fn send(&self) {
        let email = Message::builder()
            .from("henrique.ribeiro.dias@gmail.com".parse().unwrap())
            .to("henrique.ribeiro.dias@gmail.com".parse().unwrap())
            .subject("Test email from Store")
            .singlepart(SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body("Hello, this is a test email sent from a test store!".to_string()))
            .unwrap();

        // Set up the SMTP client
        let credentials = Credentials::new(self.username.clone(), self.password.clone());

        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&self.server)
            .unwrap()
            .credentials(credentials)
            .build();

        // Send the email
        match mailer.send(email).await {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => eprintln!("Failed to send email: {:?}", e),
        }
    }

    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {

        // https://myaccount.google.com/apppasswords
        SMTP {
            username: "henrique.ribeiro.dias@gmail.com".to_string(),
            password: "zszfozsnuxoeohoz".to_string(),
            server: "smtp.gmail.com".to_string(),
        }
    }
}
