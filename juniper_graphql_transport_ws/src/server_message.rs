use juniper::{ExecutionResult, GraphQLError, Variables};

use serde::Serialize;

/// ServerMessage defines the message types that server can send
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ServerMessage<S> {
    /// ConnectionAck is set in response to a client's ConnectionInit message
    /// if the server accept the connection
    ConnectionAck {
        /// Optional payload field to transfer additional
        /// details about the connection.
        payload: Option<Variables<S>>,
    },
    /// Ping message is a message sent from client and server in order to diagnostic
    /// the connection Useful for detecting failed connections, displaying latency
    /// metrics or other types of network probing.
    Ping {
        /// Optional payload field can be used to transfer additional details about the ping.
        payload: Option<Variables<S>>,
    },
    /// Pong message is send in response to a server or client Ping message to notify
    /// the counterpart that the connection is sane.
    Pong {
        /// Optional payload field can be used to transfer additional details about the pong.
        payload: Option<Variables<S>>,
    },
    /// Next message is send to a client to a client's Subscribe message in order to
    /// communicate the exectution result of a source stream created by the client.
    Next {
        /// unique operation id.
        id: String,
        /// Execution result payload.
        payload: ExecutionResult,
    },
    /// Error message is send to a client's Subscribe message if an error occurs during the
    /// varilidation process or exectution process.
    Error {
        /// unique operation id.
        id: String,
        /// payload message
        payload: Vec<GraphQLError>,
    },
    /// Complete message is a birectional message and it is send to the client when the client
    /// considerer the connection done, and want terminate the subscription. For the server instead
    /// is sent when a request of exectution completed.
    Complete {
        /// unique operatio id
        id: String,
    },
}
