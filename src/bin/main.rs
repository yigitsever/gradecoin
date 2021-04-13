use std::env;
use warp::Filter;

// use gradecoin::error;
use gradecoin::routes::consensus_routes;
use gradecoin::schema::create_database;

// mod validators;

#[tokio::main]
async fn main() {
    // Show debug logs by default by setting `RUST_LOG=gradecoin=debug`
    // TODO: write logs to file? <13-04-21, yigit> //
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "gradecoin=debug");
    }
    pretty_env_logger::init();

    let db = create_database();

    let api = consensus_routes(db);

    let routes = api.with(warp::log("gradecoin"));

    // Start the server
    let point = ([127, 0, 0, 1], 8080);
    warp::serve(routes).run(point).await;
}
