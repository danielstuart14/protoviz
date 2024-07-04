use axum::{http::StatusCode, response::{Html, Result}, routing::post, Json, Router};
use protoviz::descriptor::ProtoDescriptor;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", post(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    Json(descriptor): Json<ProtoDescriptor>,
) -> Result<Html<String>, StatusCode> {
    let result = match protoviz::render(&descriptor) {
        Ok(result) => result,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(match e {
                protoviz::errors::Error::FormatError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        },
    };

    Ok(Html(result))
}