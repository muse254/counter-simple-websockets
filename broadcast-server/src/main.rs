use aper::StateMachine;
use serde_json::{self, json};
use simple_websockets::{Event, Message, Responder};
use state::{self, CounterTransition};
use std::collections::HashMap;

fn main() {
    // Create the source of truth for all clients
    let mut counter = state::Counter::new();

    // listen for WebSockets on port 8080:
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");

    // map between client ids and the client's `Responder`:
    let mut clients: HashMap<u64, Responder> = HashMap::new();

    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);

                // Send the current state.
                // FIXME: for now the sent state is ignored.
                let _ = &responder.send(Message::Text(json!(counter.clone()).to_string()));

                // add their Responder to our `clients` map:
                clients.insert(client_id, responder);
            }
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                // remove the disconnected client from the clients map:
                clients.remove(&client_id);
            }
            Event::Message(client_id, message) => {
                // declare client.
                println!("Received a message from client #{}", client_id);

                // retrieve this client's `Responder`:
                let responder = clients.get(&client_id).unwrap();

                // unmarshal request from the client.
                match &message {
                    Message::Text(msg) => {
                        transition_and_broadcast(
                            client_id,
                            &clients,
                            &mut counter,
                            msg.to_string(),
                        );
                    }
                    _ => {
                        // declare message format expected was text.
                        println!("Message format expected was text; Client #{}", client_id);

                        // send error response
                        responder
                            .send(Message::Text("Message format expected was text".to_owned()));
                    }
                }

                // echo the message back:
                responder.send(message);
            }
        }
    }
}

/// Parses the message string and asserts whether it's a json string ready to be parsed to
/// the CounterTransition or just a regular message string and writes it out to the standard
/// output.
fn transition_and_broadcast(
    client_sender_id: u64,
    clients: &HashMap<u64, Responder>,
    state: &mut state::Counter,
    message: String,
) {
    match serde_json::from_str::<CounterTransition>(message.as_str()) {
        Ok(transition) => {
            // The transition is applied to the state.
            match state.apply(transition) {
                Ok(_) => {
                    // Messages are broadcasted to clients in lockstep.
                    clients.into_iter().for_each(|(_, responder)| {
                        responder.send(Message::Text(json!(state.clone()).to_string()));
                    });
                }
                Err(_) => {
                    // declare an error occurred when applying transition.
                    println!(
                        "An error occured applying the transition for Client #{}",
                        client_sender_id
                    );
                }
            }
        }
        Err(_) => {
            // The message sent was not a transition event.
            // declare message sent.
            println!(
                "Received a message from client #{}: {:?}",
                client_sender_id, message
            );
        }
    }
}
