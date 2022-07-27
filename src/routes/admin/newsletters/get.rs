use actix_web::{HttpResponse, http::header::ContentType};

pub async fn get_newsletter_form() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"<!DOCTYPE html><html lang="en">
        <head>    
            <meta http-equiv="content-type" content="text/html; charset=utf-8">    
            <title>Publish Newsletter</title>
        </head>
        <body>
            <form action="/admin/newsletters" method="post">
                <label>Title
                    <input type="text" placeholder="Welcome to the First Issue!" name="title">
                </label>
                <br/>
                <label>HTML Content
                    <textarea placeholder="<h1>Newsletter content</h1>" name="html_content">
                </label>
                <br/>
                <label>Plain Text Content
                    <textarea placehold="Newsletter content" name="text_content">
                </label>
                <input type="submit">Send Now!</input>
            </form>
        </body>
        </html>
        "#
    ))
}