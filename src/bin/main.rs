use gradecoin::routes::consensus_routes;
use gradecoin::schema::create_database;

#[tokio::main]
async fn main() {
    log4rs::init_file("log.conf.yml", Default::default()).unwrap();

    let db = create_database();

    let api = consensus_routes(db);

    // Start the server
    let point = ([127, 0, 0, 1], 8080);
    warp::serve(api).run(point).await;
}
