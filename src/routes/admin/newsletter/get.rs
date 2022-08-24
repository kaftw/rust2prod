use std::fmt::Write;
use actix_web::{HttpResponse, http::header::ContentType};

pub async fn get_newsletter_form(
    flash_messages: actix_web_flash_messages::IncomingFlashMessages
) -> Result<HttpResponse, actix_web::Error> {
    let mut notification_html = String::new();
    for m in flash_messages.iter() {
        writeln!(notification_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    
    let idempotency_key = uuid::Uuid::new_v4();
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        format!(r#"<!DOCTYPE html><html lang="en">
        <head>    
            <meta http-equiv="content-type" content="text/html; charset=utf-8">    
            <title>Publish Newsletter</title>
        </head>
        <body>
            {notification_html}
            <form action="/admin/newsletters" method="post">
                <label>Title
                    <input type="text" placeholder="Welcome to the First Issue!" name="title">
                </label>
                <br>
                <label>HTML Content
                    <textarea placeholder="<h1>Newsletter content</h1>" name="html_content">
                </label>
                <br>
                <label>Plain Text Content
                    <textarea placeholder="Newsletter content" name="text_content">
                </label>
                <input hidden type="text" name="idemopotency_key" value="{idempotency_key}">
                <button type="submit">Publish</button>
            </form>
        </body>
        </html>
        "#)
    ))
}