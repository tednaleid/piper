// use piper::args::Args;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::{Client, Url};
use std::io::{self, BufRead, BufReader, Stdin};
use tokio::sync::mpsc::{self, Sender};
use tokio::{runtime, task};
use tokio::time::{delay_for, Duration};
use tokio::macros::support::Future;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    // plays into the max number of concurrent requests (one being sent, N in the channel, and M in the buffer_unordered on the other end)
    // the unordered buffer in the response awaiter also plays into it
    let (mut request_tx, mut request_rx) = mpsc::channel(16);

    let client = Client::builder().gzip(true).build()?;

    let request_worker = tokio::spawn(async move {
        for i in 1..=10 {
            let resp = task::spawn(request_localhost(i, client.clone()));
            // let resp = request_localhost(i);
            if let Err(_) = request_tx.send(resp).await {
                println!("receiver dropped");
                return;
            }
        }
    });

    let response_awaiter = tokio::spawn(async move {
        let mut bu = request_rx.buffer_unordered(20);

        while let Some(handle) = bu.next().await {}
    });

    let _ = request_worker.await;
    let _ = response_awaiter.await;

    Ok(())
}

fn main() -> Result<()> {
    // let args = Args::parse();
    // let Args { input } = args;
    //
    // rt.block_on(async {
    //     let (message_sender, message_receiver) = mpsc::channel::<RequestContext>(128);
    //
    //     // doesn't work fails because dyn BufRead doesn't support Send
    //     //
    //     // let reader: Box<dyn BufRead> = if !input.is_empty() {
    //     //     Box::new(BufReader::new(File::open(input).unwrap()))
    //     // } else {
    //     //     Box::new(BufReader::new(io::stdin()))
    //     // };
    //
    //     // TODO try the tokio io aware version of BufRead to see if that works for making the input generic
    //
    //     let reader: Box<BufReader<Stdin>> = Box::new(BufReader::new(io::stdin()));
    //
    //     let worker = spawn_request_context_worker(message_sender, reader);
    //     let worker_handle = spawn_worker(message_receiver);
    //
    //     let _ = worker.await;
    //     let _ = worker_handle.await;
    // });
    //
    // Ok(())

    // todo exit with error exit code when this happens
    // match rt.block_on(future).is_err() {
    //     Err(e) => eprintln!("An error occurred: {}", e),
    // };
    //
    // Ok(())
    let future = app();
    let mut rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()?;

    // match rt.block_on(future) {
    //     Ok(_) => println!("Done"),
    //     Err(e) => println!("An error occurred: {}", e),
    // };
    //
    // Ok(())

    rt.block_on(future)
}

fn localhost(counter: u32) -> Url {
    let url = format!("http://localhost:1323/{}", counter,);
    Url::parse(&url).unwrap()
}

async fn request_localhost(id: u32, client: Client) -> Result<()> {
    let start = std::time::Instant::now();
    let url = localhost(id);
    let response = client.get(url).send().await?;

    // dummy latency on some subset of requests
    // if id % 2 == 0 {
    //     delay_for(Duration::from_millis(100)).await;
    // }

    // set RUST_LOG=info env variable to see this
    let elapsed = start.elapsed().as_secs_f32();
    println!(
        "id {} - got response {} - {:.03} => {:?}",
        id,
        response.status(),
        elapsed,
        std::thread::current().id()
    );
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
