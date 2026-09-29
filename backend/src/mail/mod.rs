//! Envoi des emails transactionnels (SMTP ou simple log en développement).

pub mod templates;

use anyhow::Result;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub use templates::EmailContent;

use crate::config::{Config, MailTransport};

#[derive(Clone)]
pub struct Mailer {
    from: Mailbox,
    transport: Transport,
    pub site_url: String,
    pub site_name: String,
}

#[derive(Clone)]
enum Transport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Log,
}

impl Mailer {
    pub fn new(config: &Config) -> Result<Self> {
        let transport = match &config.mail_transport {
            MailTransport::Log => Transport::Log,
            MailTransport::Smtp { host, port, username, password, starttls } => {
                let mut builder = if *starttls {
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)?
                } else {
                    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                };
                builder = builder.port(*port);
                if let (Some(user), Some(pass)) = (username, password) {
                    builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
                }
                Transport::Smtp(builder.build())
            }
        };
        Ok(Self {
            from: config.mail_from.parse()?,
            transport,
            site_url: config.site_url.clone(),
            site_name: config.site_name.clone(),
        })
    }

    pub async fn send(&self, to: &str, content: EmailContent) -> Result<()> {
        match &self.transport {
            Transport::Log => {
                tracing::info!(
                    to,
                    subject = %content.subject,
                    "📧 email (MAIL_TRANSPORT=log)\n{}",
                    content.text
                );
                Ok(())
            }
            Transport::Smtp(smtp) => {
                let message = Message::builder()
                    .from(self.from.clone())
                    .to(to.parse()?)
                    .subject(&content.subject)
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(content.text),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(content.html),
                            ),
                    )?;
                smtp.send(message).await?;
                Ok(())
            }
        }
    }

    /// Envoie un email en tâche de fond : la requête HTTP n'attend pas le serveur SMTP.
    pub fn send_in_background(&self, to: String, content: EmailContent) {
        let mailer = self.clone();
        tokio::spawn(async move {
            if let Err(err) = mailer.send(&to, content).await {
                tracing::error!(%to, error = ?err, "échec de l'envoi d'email");
            }
        });
    }

    pub fn link(&self, path_and_query: &str) -> String {
        format!("{}{}", self.site_url, path_and_query)
    }
}
