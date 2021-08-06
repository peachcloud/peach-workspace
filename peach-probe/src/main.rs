use log::info;

use clap::arg_enum;
use structopt::StructOpt;

mod error;
mod probe;
mod vars;

use crate::probe::PeachProbe;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "peach-probe",
    about = "a CLI tool for contract testing of the public API's exposed by PeachCloud microservices"
)]
struct Opt {
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(possible_values = &Microservice::variants(), case_insensitive = true)]
    services: Vec<Microservice>,
}

arg_enum! {
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    #[allow(clippy::enum_variant_names)]
    pub enum Microservice {
        Peach_Oled,
        Peach_Network,
        Peach_Stats,
        Peach_Menu,
        Peach_Web,
        Peach_Buttons
    }
}

impl Microservice {
    /// get_package_name converts the microservice enum to a string representation
    /// which can be used by systemctl and other tools which reference the package by name
    /// we can't use std::fmt::Display because this is already used by arg_enum!
    pub fn get_package_name(service: &Microservice) -> String {
        let s = match service {
            Microservice::Peach_Oled => "peach-oled",
            Microservice::Peach_Network => "peach-network",
            Microservice::Peach_Stats => "peach-stats",
            Microservice::Peach_Menu => "peach-menu",
            Microservice::Peach_Web => "peach-web",
            Microservice::Peach_Buttons => "peach-buttons",
        };
        s.to_string()
    }
}

fn main() {
    // initialize the logger
    env_logger::init();

    // parse cli arguments
    let opt = Opt::from_args();

    // debugging what was parsed
    info!("probing services: {:?}", opt.services);
    if opt.verbose {
        info!("using verbose mode")
    }

    let services;
    // if not arguments were provided, then we probe all services
    if opt.services.is_empty() {
        services = vec![
            Microservice::Peach_Network,
            Microservice::Peach_Oled,
            Microservice::Peach_Stats,
            Microservice::Peach_Web,
            Microservice::Peach_Buttons,
            Microservice::Peach_Menu,
        ]
    } else {
        services = opt.services;
    }

    // instantiate the probe
    let mut probe: PeachProbe = PeachProbe::new(opt.verbose);

    // iterate through services and run probe tests on them
    for service in services {
        probe.probe_service(service);
    }

    // final report of how many microservices returned successes and failures
    println!("[ generating report ]");
    for result in probe.results {
        let num_failures = result.failures.len();
        let report;
        // if service is running according to systemctl status
        if result.is_running {
            if num_failures == 0 {
                report = format!(
                    "- {} [version: {}] is online.",
                    result.microservice, result.version
                );
                println!("{}", report);
            }
            // even if its running, some endpoints could still return errors
            else {
                report = format!(
                    "- {} [version: {}] is online but {} endpoints returned errors: {:?}",
                    result.microservice, result.version, num_failures, result.failures
                );
                eprintln!("{}", report);
            }
        }
        // if service is not running according to systemctl status, print the service log
        else {
            match result.service_log {
                Some(service_log) => {
                    report = format!(
                        "- {} [version: {}] is offline, with log:\n {}",
                        result.microservice, result.version, service_log
                    );
                }
                None => {
                    report = format!(
                        "- {} [version: {}] is offline, log not found",
                        result.microservice, result.version
                    );
                }
            };
            eprintln!("{}", report);
        }
    }
}
