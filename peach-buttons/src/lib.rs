mod error;
mod interrupt;

use std::{env, result::Result, sync::Arc, thread};

use crossbeam_channel::bounded;
use jsonrpc_core::futures::Future;
use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
#[allow(unused_imports)]
use jsonrpc_test as test;
use jsonrpc_ws_server::{RequestContext, ServerBuilder};
use log::{debug, error, info, warn};

use crate::error::{BoxError, ButtonError::RejectSubscription};
use crate::interrupt::*;

pub fn run() -> Result<(), BoxError> {
    info!("Starting up.");

    debug!("Creating channel for message passing.");
    let (s, r) = bounded(0);

    let pin = vec![4, 27, 23, 17, 22, 5, 6];
    let code = vec![0, 1, 2, 3, 4, 5, 6];
    let name = vec!["center", "left", "right", "up", "down", "#5", "#6"];

    debug!("Setting up interrupt handlers.");
    for i in 0..7 {
        interrupt_handler(pin[i], code[i], name[i].to_string(), s.clone());
    }

    debug!("Creating pub-sub handler.");
    let mut io = PubSubHandler::new(MetaIoHandler::default());

    io.add_subscription(
        "button_press",
        (
            "subscribe_buttons",
            move |params: Params, _, subscriber: Subscriber| {
                debug!("Received subscription request.");
                if params != Params::None {
                    subscriber
                        .reject(Error::from(RejectSubscription))
                        .unwrap_or_else(|_| {
                            error!("Failed to send rejection error for subscription request.");
                        });
                    return;
                }

                let r1 = r.clone();

                thread::spawn(move || {
                    let sink = subscriber
                        .assign_id_async(SubscriptionId::Number(1))
                        .wait()
                        .unwrap();

                    info!("Listening for button code from gpio events.");
                    loop {
                        let button_code: u8 = r1.recv().unwrap();
                        info!("Received button code: {}.", button_code);
                        match sink
                            .notify(Params::Array(vec![Value::Number(button_code.into())]))
                            .wait()
                        {
                            Ok(_) => info!("Publishing button code to subscriber over ws."),
                            Err(_) => {
                                warn!("Failed to publish button code.");
                                break;
                            }
                        }
                    }
                });
            },
        ),
        ("remove_buttons", |_id: SubscriptionId, _| {
            // unsubscribe
            futures::future::ok(Value::Bool(true))
        }),
    );

    let ws_server =
        env::var("PEACH_BUTTONS_SERVER").unwrap_or_else(|_| "127.0.0.1:5111".to_string());

    info!("Starting JSON-RPC server on {}.", ws_server);
    let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
        Arc::new(Session::new(context.sender()))
    })
    .start(
        &ws_server
            .parse()
            .expect("Invalid WS address and port combination"),
    )
    .expect("Unable to start RPC server");

    info!("Listening for requests.");
    server.wait().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
