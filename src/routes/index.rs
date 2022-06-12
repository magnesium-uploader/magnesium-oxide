use actix_web::{Error, HttpRequest, HttpResponse, Result};
use futures_util::StreamExt;

use crate::{structs::files::File, AppState};

/// Frontend index route
pub async fn index(request: HttpRequest) -> Result<HttpResponse, Error> {
    let state = request.app_data::<AppState>().unwrap();

    // Get all file sizes from the database and add them up
    let mut total_size: i64 = 0;
    let mut total_files: i64 = 0;

    {
        let files = state.database.collection::<File>("files");
        let mut cursor = files.find(None, None).await.unwrap();

        // TODO: Cache these results in a `meta` collection in the database.
        while let Some(file) = cursor.next().await {
            let file = file.unwrap();

            total_size += file.size;
            total_files += 1;
        }
    }

    Ok(HttpResponse::Ok().content_type("text/html").body(format!(
        r#"<html lang="en" data-theme="dark">
            <head>
                <title>mgo.li</title>
                <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/minstyle.io@2.0.1/dist/css/minstyle.io.min.css">
            </head>
            <body>
                <div class="container">
                    <h1>mgo.li</h1>
                    <p> This server is running Magnesium-oxide v<b>{}</b> </p>
                    <p> We are currently hosting <b>{}</b> files totalling <b>{}</b> MB. </p>
                    <p> Intrested in creating an account? Join our <a href="https://discord.gg/GHvGtq9xPB">Discord</a>! </p>
                </div>
            </body>
        </html>"#,
        env!("CARGO_PKG_VERSION"),
        total_files,
        total_size / 1024 / 1024
    )))
}
