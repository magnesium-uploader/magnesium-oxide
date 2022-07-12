use actix_web::{Error, HttpRequest, HttpResponse, Result};
use futures_util::StreamExt;

use crate::{structs::files::File, AppState};
use tera::Context;

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

    let mut context = Context::new();
    context.insert("total_size", &total_size);
    context.insert("total_files", &total_files);
    context.insert("version", env!("CARGO_PKG_VERSION"));

    let html = state.tera.render("index.html", &context).unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
