fn main() {
    dotenv::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("`DATABASE_URL` must be set");

    let config = controllers::config::get_config();

    controllers::start_dusty_server(url, config);
}
