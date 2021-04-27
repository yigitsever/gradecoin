use std::env;

// use gradecoin::error;
use gradecoin::routes::consensus_routes;
use gradecoin::schema::create_database;

// mod validators;

#[tokio::main]
async fn main() {
    // Show debug logs by default by setting `RUST_LOG=gradecoin=debug`
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "gradecoin=debug");
    }
    log4rs::init_file("log.conf.yml", Default::default()).unwrap();

    let db = create_database();

    let api = consensus_routes(db);

    // let routes = api.with(warp::log("gradecoin"));

    // Start the server
    let point = ([127, 0, 0, 1], 8080);
    warp::serve(api).run(point).await;
}
