//! Gabarits des emails (texte + HTML aux couleurs du site).

use ammonia::clean_text;

pub struct EmailContent {
    pub subject: String,
    pub text: String,
    pub html: String,
}

fn layout(site_name: &str, title: &str, body_html: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="fr"><body style="margin:0;background:#f9fafe;font-family:'Open Sans',Arial,sans-serif;color:#343434">
<table width="100%" cellpadding="0" cellspacing="0" style="padding:32px 12px"><tr><td align="center">
<table width="560" cellpadding="0" cellspacing="0" style="max-width:560px;background:#ffffff;border-radius:6px;overflow:hidden">
<tr><td style="background:linear-gradient(135deg,#19b1f0,#0171cd);background-color:#0171cd;padding:24px 32px;color:#fff;font-family:Lato,Arial,sans-serif;font-weight:900;font-size:22px;letter-spacing:2px">GP · {site}</td></tr>
<tr><td style="padding:32px">
<h1 style="font-family:Lato,Arial,sans-serif;font-size:22px;margin:0 0 16px;color:#242424">{title}</h1>
{body}
</td></tr>
<tr><td style="padding:16px 32px;background:#f9fafe;color:#8a8a8a;font-size:12px">Vous recevez cet email car une action a été effectuée avec votre adresse sur {site}.</td></tr>
</table></td></tr></table></body></html>"#,
        site = clean_text(site_name),
        title = clean_text(title),
        body = body_html,
    )
}

fn button(url: &str, label: &str) -> String {
    format!(
        r#"<p style="margin:28px 0"><a href="{url}" style="display:inline-block;padding:12px 28px;border-radius:500px;background:#32beff;color:#fff;text-decoration:none;font-weight:700;text-transform:uppercase;letter-spacing:1px;font-size:13px">{label}</a></p>
<p style="font-size:13px;color:#8a8a8a">Si le bouton ne fonctionne pas, copiez ce lien : <br><a href="{url}" style="color:#0171cd;word-break:break-all">{url}</a></p>"#,
        url = clean_text(url),
        label = clean_text(label),
    )
}

fn action_email(site: &str, subject: &str, intro: &str, url: &str, label: &str, validity: &str) -> EmailContent {
    let html = layout(
        site,
        subject,
        &format!(
            "<p>{}</p>{}<p style=\"font-size:13px\">Ce lien est valable <strong>{}</strong> et ne peut être utilisé qu'une seule fois.</p>",
            clean_text(intro),
            button(url, label),
            clean_text(validity)
        ),
    );
    EmailContent {
        subject: format!("{subject} — {site}"),
        text: format!("{intro}\n\n{label} : {url}\n\nCe lien est valable {validity} et ne peut être utilisé qu'une seule fois."),
        html,
    }
}

pub fn verify_account(site: &str, url: &str) -> EmailContent {
    action_email(
        site,
        "Confirmez votre adresse email",
        "Bienvenue ! Confirmez votre adresse email pour activer votre compte. Vous pourrez ensuite définir votre mot de passe.",
        url,
        "Activer mon compte",
        "24 heures",
    )
}

pub fn verify_subscription(site: &str, url: &str) -> EmailContent {
    action_email(
        site,
        "Confirmez votre inscription à la newsletter",
        "Merci pour votre inscription ! Confirmez votre adresse email pour recevoir les nouveaux articles et accéder à votre espace abonné.",
        url,
        "Confirmer mon inscription",
        "24 heures",
    )
}

pub fn magic_link(site: &str, url: &str) -> EmailContent {
    action_email(
        site,
        "Votre lien de connexion",
        "Voici votre lien de connexion sans mot de passe.",
        url,
        "Me connecter",
        "10 minutes",
    )
}

pub fn reset_password(site: &str, url: &str) -> EmailContent {
    action_email(
        site,
        "Modification de votre mot de passe",
        "Une demande de modification de mot de passe a été effectuée pour votre compte. Si vous n'êtes pas à l'origine de cette demande, ignorez cet email.",
        url,
        "Définir un nouveau mot de passe",
        "10 minutes",
    )
}

pub fn new_article(site: &str, title: &str, excerpt: &str, url: &str) -> EmailContent {
    let html = layout(
        site,
        title,
        &format!(
            "<p>Un nouvel article vient d'être publié :</p><p style=\"color:#555\">{}</p>{}",
            clean_text(excerpt),
            button(url, "Lire l'article")
        ),
    );
    EmailContent {
        subject: format!("Nouvel article : {title}"),
        text: format!("Un nouvel article vient d'être publié : {title}\n\n{excerpt}\n\nLire l'article : {url}"),
        html,
    }
}
