use peach_lib::dyndns_client::{dyndns_update_ip, register_domain, is_dns_updater_online, log_successful_nsupdate, get_num_seconds_since_successful_dns_update };
use peach_lib::password_utils::{verify_password, set_new_password, verify_temporary_password, set_new_temporary_password, send_password_reset};
use peach_lib::config_manager::{add_ssb_admin_id, delete_ssb_admin_id};
use peach_lib::sbot_client;
use std::process;
use chrono::prelude::*;


fn main() {
    // initalize the logger
    env_logger::init();
//
//    println!("Hello, world its debug!");
//    let result = set_new_password("password3");
//    println!("result: {:?}", result);
//
//    let result = verify_password("password1");
//    println!("result should be error: {:?}", result);
//
//    let result = verify_password("password3");
//    println!("result should be ok: {:?}", result);
//
//
//    println!("Testing temporary passwords");
//    let result = set_new_temporary_password("abcd");
//    println!("result: {:?}", result);
//
//    let result = verify_temporary_password("password1");
//    println!("result should be error: {:?}", result);
//
//    let result = verify_temporary_password("abcd");
//    println!("result should be ok: {:?}", result);
//
    let result = send_password_reset();
    println!("send password reset result should be ok: {:?}", result);

//    sbot_client::post("hi cat");
//    let result = sbot_client::whoami();
//        let result = sbot_client::create_invite(50);
//        let result = sbot_client::post("is this working");
//    println!("result: {:?}", result);
//        let result = sbot_client::post("nice we have contact");
//        let result = sbot_client::update_pub_name("vermont-pub");
//        let result = sbot_client::private_message("this is a private message", "@LZx+HP6/fcjUm7vef2eaBKAQ9gAKfzmrMVGzzdJiQtA=.ed25519");
//        println!("result: {:?}", result);

//   let result = send_password_reset();
//    let result = add_ssb_admin_id("xyzdab");
//   println!("result: {:?}", result);
//    let result = delete_ssb_admin_id("xyzdab");
//    println!("result: {:?}", result);
//    let result = delete_ssb_admin_id("ab");
//    println!("result: {:?}", result);

////    let result = log_successful_nsupdate();
////    let result = get_num_seconds_since_successful_dns_update();
//    let is_online = is_dns_updater_online();
//    println!("is online: {:?}", is_online);
//
////    let result = get_last_successful_dns_update();
////    println!("result: {:?}", result);
////    register_domain("newquarter299.dyn.peachcloud.org");
//    let result = dyndns_update_ip();
//    println!("result: {:?}", result);
}
