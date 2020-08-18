use piper::args::Args;
use reqwest::Client;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let Args { input } = args;

    let (mut message_sender, message_receiver) = mpsc::channel::<RequestContext>(128);

    let worker_handle = spawn_worker(message_receiver);

    let reader: Box<dyn BufRead> = if !input.is_empty() {
        Box::new(BufReader::new(File::open(input).unwrap()))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    for url_result in reader.lines() {
        let request_context = RequestContext {
            url: url_result.unwrap(),
        };

        // eprintln!("before send url: {}", request_context.url);
        if message_sender.send(request_context).await.is_err() {
            break;
        }
    }

    let _ = worker_handle.await;
}

fn spawn_worker(mut receiver: mpsc::Receiver<RequestContext>) -> JoinHandle<()> {
    let client = Client::new();
    tokio::spawn(async move {
        while let Some(request_context) = receiver.recv().await {
            let url = request_context.url;
            let response = client.get(&url).send().await.unwrap();
            println!("result {} - {}", response.url(), response.status())
        }
    })
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
