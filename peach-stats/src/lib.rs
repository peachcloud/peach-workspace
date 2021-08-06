mod error;
mod stats;
mod structs;

use std::{env, result::Result};

use jsonrpc_core::{IoHandler, Value};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, ServerBuilder};
#[allow(unused_imports)]
use jsonrpc_test as test;
use log::info;

use crate::error::BoxError;

pub fn run() -> Result<(), BoxError> {
    info!("Starting up.");

    info!("Creating JSON-RPC I/O handler.");
    let mut io = IoHandler::default();

    io.add_method("cpu_stats", move |_| {
        info!("Fetching CPU statistics.");
        let stats = stats::cpu_stats()?;

        Ok(Value::String(stats))
    });

    io.add_method("cpu_stats_percent", move |_| {
        info!("Fetching CPU statistics as percentages.");
        let stats = stats::cpu_stats_percent()?;

        Ok(Value::String(stats))
    });

    io.add_method("disk_usage", move |_| {
        info!("Fetching disk usage statistics.");
        let disks = stats::disk_usage()?;

        Ok(Value::String(disks))
    });

    io.add_method("load_average", move |_| {
        info!("Fetching system load average statistics.");
        let avg = stats::load_average()?;

        Ok(Value::String(avg))
    });

    io.add_method("mem_stats", move |_| {
        info!("Fetching current memory statistics.");
        let mem = stats::mem_stats()?;

        Ok(Value::String(mem))
    });

    io.add_method("ping", |_| Ok(Value::String("success".to_string())));

    io.add_method("uptime", move |_| {
        info!("Fetching system uptime.");
        let uptime = stats::uptime()?;

        Ok(Value::String(uptime))
    });

    let http_server = env::var("PEACH_OLED_STATS").unwrap_or_else(|_| "127.0.0.1:5113".to_string());

    info!("Starting JSON-RPC server on {}.", http_server);
    let server = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
        ]))
        .start_http(
            &http_server
                .parse()
                .expect("Invalid HTTP address and port combination"),
        )
        .expect("Unable to start RPC server");

    info!("Listening for requests.");
    server.wait();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // test to ensure correct success response
    #[test]
    fn rpc_success() {
        let rpc = {
            let mut io = IoHandler::new();
            io.add_method("rpc_success_response", |_| {
                Ok(Value::String("success".into()))
            });
            test::Rpc::from(io)
        };

        assert_eq!(rpc.request("rpc_success_response", &()), r#""success""#);
    }
}
