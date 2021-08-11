/*
Broadcast Channel del crate Tokio
Because these challenges are common when implementing many-to-many communi‐
cation patterns, the tokio crate provides a broadcast channel type that implements
one reasonable set of tradeoffs. A tokio broadcast channel is a queue of values (in
our case, chat messages) that allows any number of different threads or tasks to send
and receive values. It’s called a “broadcast” channel because every consumer gets its
own copy of each value sent. (The value type must implement Clone .)

Normally, a broadcast channel retains a message in the queue until every consumer
has gotten their copy. But if the length of the queue would exceed the channel’s maxi‐
mum capacity, specified when it is created, the oldest messages get dropped. Any con‐
sumers who couldn’t keep up get an error the next time they try to get their next
message, and the channel catches them up to the oldest message still available.

*/

use async_std::task;
use crate::connection::Outbound;
use std::sync::Arc;
use tokio::sync::broadcast;

// Group type definition
pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>
}

impl Group {
    // Lo crea
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        Group { name, sender }
    }

    // Une a un usuario al grupo
    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();

        task::spawn(handle_subscriber(self.name.clone(),
                                     receiver,
                                     outbound));
    }

    // Envía un mensaje al grupo usando el broadcast channel (sender)
    pub fn post(&self, message: Arc<String>) {
    // This only returns an error when there are no subscribers. A
    // connection's outgoing side can exit, dropping its subscription,
    // slightly before its incoming side, which may end up trying to send a
    // message to an empty group.

        let _ignored = self.sender.send(message);
    }
}

use async_chat::FromServer;
use tokio::sync::broadcast::error::RecvError;

async fn handle_subscriber(group_name: Arc<String>,
                           mut receiver: broadcast::Receiver<Arc<String>>,
                           outbound: Arc<Outbound>)
{
    loop {
        // Recibe mensajes y los transmite al cliente usando el Outbound value
        let packet = match receiver.recv().await {
            Ok(message) => FromServer::Message {
                group_name: group_name.clone(),
                message: message.clone(),
            },
            Err(RecvError::Lagged(n)) => FromServer::Error(
                format!("Dropped {} messages from {}.", n, group_name)
            ),
            Err(RecvError::Closed) => break,
        };
        if outbound.send(packet).await.is_err() {
            break;
        }
    }
}