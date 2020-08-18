use piper::args::Args;
use reqwest::Client;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Result};
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

fn main() -> Result<()> {
    let args = Args::parse();

    let Args { input } = args;

    let mut runtime = Builder::new()
        .threaded_scheduler()
        .core_threads(100)
        .max_threads(1000)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let reader: Box<dyn BufRead> = if !input.is_empty() {
            Box::new(BufReader::new(File::open(input).unwrap()))
        } else {
            Box::new(BufReader::new(io::stdin()))
        };

        let (mut message_sender, message_receiver) = mpsc::channel::<RequestContext>(128);

        let worker_handle = spawn_worker(message_receiver);

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
    });

    Ok(())
}

fn spawn_worker(mut receiver: mpsc::Receiver<RequestContext>) -> JoinHandle<()> {
    let client = Client::new();
    tokio::spawn(async move {
        println!("inside spawn!");
        loop {
            let request_context = receiver.recv().await.unwrap();
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
