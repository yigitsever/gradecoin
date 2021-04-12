use gradecoin::schema::create_database;
use std::env;
use warp::Filter;

mod custom_filters;
mod handlers;
mod routes;
// mod validators;

#[tokio::main]
async fn main() {
    // Show debug logs by default by setting `RUST_LOG=restful_rust=debug`
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "gradecoin=debug");
    }
    pretty_env_logger::init();

    let db = create_database();

    let api = routes::consensus_routes(db);

    let routes = api.with(warp::log("gradecoin"));

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
