use peach_lib::dyndns_client::dyndns_update_ip;
use log::{info};


fn main() {
    // initalize the logger
    env_logger::init();

    info!("Running peach-dyndns-updater");
    let result = dyndns_update_ip();
    info!("result: {:?}", result);
}