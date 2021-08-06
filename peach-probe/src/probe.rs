use peach_lib::error::PeachError;
use peach_lib::network_client;
use peach_lib::oled_client;
use peach_lib::stats_client;

use log::info;
use regex::Regex;
use std::process::Command;

use crate::error::ProbeError;
use crate::vars::PEACH_LOGO;
use crate::Microservice;

/// ProbeResult stores the results of probing a particular microservice
pub struct ProbeResult {
    // string of the name of the service
    pub microservice: String,
    // string of the version of this service currently installed
    pub version: String,
    // vector of names of endpoints which had errors
    pub failures: Vec<String>,
    // vector of names of endpoints which returned successfully
    pub successes: Vec<String>,
    // bool which stores true if the service is running
    pub is_running: bool,
    // string which stores the tail of the log from journalctl -u service
    pub service_log: Option<String>,
}

impl ProbeResult {
    fn new(microservice: &str) -> ProbeResult {
        ProbeResult {
            microservice: microservice.to_string(),
            failures: Vec::new(),
            successes: Vec::new(),
            is_running: false,
            version: "".to_string(),
            service_log: None,
        }
    }
}

/// PeachProbe implements probes for all microservices and data structures
/// for storing the results of all probes
pub struct PeachProbe {
    pub results: Vec<ProbeResult>,
    pub verbose: bool,
}

impl PeachProbe {
    pub fn new(verbose: bool) -> PeachProbe {
        PeachProbe {
            results: Vec::new(),
            verbose,
        }
    }

    /// probe any microservice, using systemctl status to see if the service is running
    /// and testing endpoints for services which support this (peach-stats, peach-network, peach-oled)
    /// for all other microservices this function just checks if the service is running
    pub fn probe_service(&mut self, service: Microservice) {
        // get package name from enum
        let service_name = Microservice::get_package_name(&service);
        println!("[ probing {} ]", service_name);

        // instantiate ProbeResult
        let mut result = ProbeResult::new(&service_name);

        // get version of service
        result.version = PeachProbe::get_service_version(&service_name);

        // check status of service
        let status_result = PeachProbe::get_service_status(&service_name);
        match status_result {
            Ok(is_running) => {
                result.is_running = is_running;
                // if the service is not running, get the journalctl log of the service
                if !is_running {
                    let log_result = PeachProbe::get_service_log(&service_name);
                    match log_result {
                        Ok(log) => {
                            result.service_log = Some(log);
                        }
                        Err(err) => {
                            eprintln!("error getting log for {}: {:#?}", service_name, err);
                        }
                    }
                }
            }
            Err(err) => {
                result.is_running = false;
                eprintln!(
                    "error retrieving service status of {}: {:#?}",
                    service_name, err
                );
            }
        }

        // probe endpoints for the serivce if applicable
        let result = match service {
            Microservice::Peach_Stats => self.peach_stats(result),
            Microservice::Peach_Oled => self.peach_oled(result),
            Microservice::Peach_Network => self.peach_network(result),
            _ => {
                info!("probing endpoints not implemented for this service");
                result
            }
        };

        // save result
        self.results.push(result);
    }

    /// helper function which gets the version of the microservice running using apt-get
    fn get_service_version_result(service: &str) -> Result<String, ProbeError> {
        let output = Command::new("/usr/bin/apt")
            .arg("list")
            .arg(service)
            .output()?;
        let command_output = std::str::from_utf8(&output.stdout)?;
        // use a regex to get the version number from the string
        let re = Regex::new(r".*buster,now (\d+\.\d+\.\d+) arm64.*")?;
        let cap = re.captures(command_output);
        match cap {
            Some(c) => {
                let version = &c[1];
                Ok(version.to_string())
            }
            None => Err(ProbeError::GetServiceVersionRegexMatchError),
        }
    }

    /// helper function to call systemctl status for service
    pub fn get_service_status(service: &str) -> Result<bool, ProbeError> {
        let output = Command::new("/usr/bin/systemctl")
            .arg("status")
            .arg(service)
            .output()?;
        let status = output.status;
        // returns true if the service had an exist status of 0 (is running)
        let is_running = status.success();
        Ok(is_running)
    }

    /// helper function to get last 2 lines of journalctl log for service
    pub fn get_service_log(service: &str) -> Result<String, ProbeError> {
        let output = Command::new("/usr/bin/journalctl")
            .arg("-u")
            .arg(service)
            .arg("-t")
            .arg(service)
            .arg("-n")
            .arg("3")
            .output()?;
        let log_output = String::from_utf8(output.stdout)?;
        Ok(log_output)
    }

    /// helper function which gets the version of the microservice running using apt-get as a string
    /// if there is an error getting the version, it returns the string "Unknown"
    fn get_service_version(service: &str) -> String {
        let version_result = PeachProbe::get_service_version_result(service);
        match version_result {
            Ok(version) => version,
            Err(_) => "Unknown".to_string(),
        }
    }

    /// helper function for probing an endpoint on a peach microservice and collecting errors for a final report
    fn probe_peach_endpoint<T>(
        &mut self,
        endpoint_result: Result<T, PeachError>,
        endpoint_name: &str,
        result: &mut ProbeResult,
    ) {
        match endpoint_result {
            Ok(_) => {
                if self.verbose {
                    println!("++ {} endpoint is online", endpoint_name);
                }
                result.successes.push(endpoint_name.to_string());
            }
            Err(e) => {
                eprintln!("++ {} endpoint is offline", endpoint_name);
                match e {
                    PeachError::JsonRpcHttp(e) => {
                        eprintln!("Returned JsonRpcHTTP error: {:#?}\n", e)
                    }
                    PeachError::JsonRpcCore(e) => {
                        eprintln!("Returned JsonRpcCore error: {:#?}\n", e)
                    }
                    PeachError::Serde(_) => eprintln!("Returned Serde Json serialization error\n"),
                }
                result.failures.push(endpoint_name.to_string());
            }
        }
    }

    /// helper function for probing an endpoint on a peach microservice which expects a particular JsonRPCCore Error
    fn probe_assert_error_endpoint<T>(
        &mut self,
        endpoint_result: Result<T, PeachError>,
        endpoint_name: &str,
        expected_error_code: i64,
        result: &mut ProbeResult,
    ) {
        match endpoint_result {
            Ok(_) => {
                eprintln!("++ this endpoint should not return successfully during peach-probe, something is strange");
                result.failures.push(endpoint_name.to_string());
            }
            Err(e) => {
                match e {
                    PeachError::JsonRpcCore(e) => {
                        match e.kind() {
                            // this is the expected error, all other errors are unexpected
                            jsonrpc_client_core::ErrorKind::JsonRpcError(err) => {
                                if err.code.code() == expected_error_code {
                                    if self.verbose {
                                        println!("++ {} endpoint is online", endpoint_name);
                                    }
                                    result.successes.push(endpoint_name.to_string());
                                } else {
                                    eprintln!("++ {} endpoint is offline", endpoint_name);
                                    eprintln!("Returned JsonRpcCore error with unexpected code or message: {:#?}\n", e);
                                    result.failures.push(endpoint_name.to_string());
                                }
                            }
                            _ => {
                                eprintln!("++ {} endpoint is offline", endpoint_name);
                                eprintln!("Returned unexpected JsonRpcCore error: {:#?}\n", e);
                                result.failures.push(endpoint_name.to_string());
                            }
                        }
                    }
                    PeachError::JsonRpcHttp(e) => {
                        eprintln!("++ {} endpoint is offline", endpoint_name);
                        eprintln!("Returned JsonRpcHTTP error: {:#?}\n", e);
                        result.failures.push(endpoint_name.to_string());
                    }
                    PeachError::Serde(_) => {
                        eprintln!("++ {} endpoint is offline", endpoint_name);
                        eprintln!("Returned Serde Json serialization error\n");
                        result.failures.push(endpoint_name.to_string());
                    }
                }
            }
        }
    }

    /// probes all endpoints on the peach-stats microservice
    pub fn peach_stats(&mut self, mut result: ProbeResult) -> ProbeResult {
        // probe endpoints
        self.probe_peach_endpoint(
            stats_client::cpu_stats_percent(),
            "cpu_stats_percent",
            &mut result,
        );
        self.probe_peach_endpoint(stats_client::load_average(), "load_average", &mut result);
        self.probe_peach_endpoint(stats_client::disk_usage(), "disk_usage", &mut result);
        self.probe_peach_endpoint(stats_client::mem_stats(), "mem_stats", &mut result);
        self.probe_peach_endpoint(stats_client::ping(), "ping", &mut result);
        self.probe_peach_endpoint(stats_client::uptime(), "uptime", &mut result);

        // save result
        result
    }

    /// probes all endpoints on peach-network microservice
    pub fn peach_network(&mut self, mut result: ProbeResult) -> ProbeResult {
        // probe endpoints which should successfully return if online
        self.probe_peach_endpoint(
            network_client::add("peach-probe-test-ssid", "peach-probe-test-pass"),
            "add",
            &mut result,
        );
        self.probe_peach_endpoint(
            network_client::available_networks("wlan0"),
            "available_networks",
            &mut result,
        );
        self.probe_peach_endpoint(
            network_client::id("wlan0", "peach-probe-test-ssid"),
            "id",
            &mut result,
        );
        self.probe_peach_endpoint(network_client::ip("wlan0"), "ip", &mut result);
        self.probe_peach_endpoint(network_client::ssid("wlan0"), "ssid", &mut result);
        self.probe_peach_endpoint(network_client::ping(), "ping", &mut result);
        self.probe_peach_endpoint(network_client::reconfigure(), "reconfigure", &mut result);
        self.probe_peach_endpoint(
            network_client::saved_networks(),
            "saved_networks",
            &mut result,
        );
        self.probe_peach_endpoint(network_client::state("wlan0"), "state", &mut result);
        self.probe_peach_endpoint(network_client::traffic("wlan0"), "traffic", &mut result);
        self.probe_peach_endpoint(
            network_client::forget("wlan0", "peach-probe-test-ssid"),
            "forget",
            &mut result,
        );

        // if online, the following functions should return an error which we should catch and confirm
        self.probe_assert_error_endpoint(
            network_client::connect("peach-probe-test-ssid", "wlan0"),
            "connect",
            -32027,
            &mut result,
        );

        // probe switching between ap and client mode
        self.probe_peach_endpoint(network_client::activate_ap(), "activate_ap", &mut result);
        self.probe_peach_endpoint(
            network_client::activate_client(),
            "activate_client",
            &mut result,
        );

        // return result
        result
    }

    /// probes all endpoints on the peach-oled microservice
    pub fn peach_oled(&mut self, mut result: ProbeResult) -> ProbeResult {
        // probe endpoints
        self.probe_peach_endpoint(oled_client::ping(), "ping", &mut result);

        // probe clear and flush
        self.probe_peach_endpoint(oled_client::clear(), "clear", &mut result);
        self.probe_peach_endpoint(
            oled_client::write(0, 0, "peach-probe success", "6x8"),
            "write",
            &mut result,
        );

        // probe draw endpoint
        let bytes = PEACH_LOGO.to_vec();
        self.probe_peach_endpoint(
            oled_client::draw(bytes, 64, 64, 32, 10),
            "draw",
            &mut result,
        );

        // just clear at the end without flush so that state of peach-oled is not changed
        self.probe_peach_endpoint(oled_client::flush(), "flush", &mut result);

        // test power off endpoint
        self.probe_peach_endpoint(oled_client::power(false), "power-off", &mut result);
        self.probe_peach_endpoint(oled_client::power(true), "power-on", &mut result);

        // return result
        result
    }
}
