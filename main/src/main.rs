fn main() {
    dotenv::dotenv().ok();

    let url = "".to_owned();

    let config = controllers::config::get_config();

    controllers::start_dusty_server(&url, config);
}
