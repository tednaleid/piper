// use piper::args::Args;
use reqwest::Client;
use std::io::{self, BufRead, BufReader, Stdin};
use tokio::sync::mpsc::{self, Sender};
use tokio::{runtime, task};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    // let args = Args::parse();
    // let Args { input } = args;

    let mut rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let (message_sender, message_receiver) = mpsc::channel::<RequestContext>(128);

        // doesn't work fails because dyn BufRead doesn't support Send
        //
        // let reader: Box<dyn BufRead> = if !input.is_empty() {
        //     Box::new(BufReader::new(File::open(input).unwrap()))
        // } else {
        //     Box::new(BufReader::new(io::stdin()))
        // };

        // TODO try the tokio io aware version of BufRead to see if that works for making the input generic

        let reader: Box<BufReader<Stdin>> = Box::new(BufReader::new(io::stdin()));

        let worker = spawn_request_context_worker(message_sender, reader);
        let worker_handle = spawn_worker(message_receiver);

        let _ = worker.await;
        let _ = worker_handle.await;
    });

    Ok(())
}

async fn spawn_request_context_worker(
    mut message_sender: Sender<RequestContext>,
    reader: Box<BufReader<Stdin>>,
) {
    task::spawn(async move {
        for url_result in reader.lines() {
            let request_context = RequestContext {
                url: url_result.unwrap(),
            };

            if message_sender.send(request_context).await.is_err() {
                return;
            }
        }
    });
}

async fn spawn_worker(mut receiver: mpsc::Receiver<RequestContext>) {
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
