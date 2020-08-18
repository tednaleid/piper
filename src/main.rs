use piper::args::Args;
use reqwest::Client;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tokio::sync::mpsc;
use tokio::task::{self, JoinHandle};
use tokio::io::Error;
use tokio::sync::mpsc::Sender;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let Args { input } = args;

    let (mut message_sender, message_receiver) = mpsc::channel::<RequestContext>(128);

    let worker_handle = spawn_worker(message_receiver);

    spawn_request_context_worker(input, &mut message_sender).await;
    // task::spawn(spawn_request_context_worker(input, &mut message_sender)).await;

    // task::spawn(async move {
    //     let url = "http://localhost:1323/100".to_string();
    //     let v = RequestContext { url };
    //     message_sender.send(v).await;
    // }).await;

    let _ = worker_handle.await;

    Ok(())
}

async fn spawn_request_context_worker(input: String, message_sender: &mut Sender<RequestContext>) -> () {
    let reader: Box<dyn BufRead> = if !input.is_empty() {
        Box::new(BufReader::new(File::open(input).unwrap()))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    for url_result in reader.lines() {
        let request_context = RequestContext {
            url: url_result.unwrap(),
        };

        if message_sender.send(request_context).await.is_err() {
            break;
        }
    }
}

async fn spawn_worker(mut receiver: mpsc::Receiver<RequestContext>) -> () {
    let client = Client::new();
    while let Some(request_context) = receiver.recv().await {
        let url = request_context.url;
        let response = client.get(&url).send().await.unwrap();
        println!("result {} - {}", response.url(), response.status())
    }
}

#[derive(Debug)]
pub struct RequestContext {
    url: String,
}

impl PartialEq for RequestContext {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
