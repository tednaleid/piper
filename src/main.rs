use futures::StreamExt;
use reqwest::{Client, Url};
use std::io::{self, BufRead, BufReader, Stdin};
use tokio::sync::mpsc::{self};
use tokio::{runtime, task};
// use tokio::time::{delay_for, Duration};
// use piper::args::Args;
// use tokio::fs::File;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    // plays into the max number of concurrent requests (one being sent, N in the channel, and M in the buffer_unordered on the other end)
    // the unordered buffer in the response awaiter also plays into it
    let (mut request_tx, request_rx) = mpsc::channel(16);

    let client = Client::builder().gzip(true).build()?;

    let reader: Box<BufReader<Stdin>> = Box::new(BufReader::new(io::stdin()));

    // let args = Args::parse();
    // let Args { input } = args;
    //
    // TODO need to figure out AsyncBufRead stuff here
    // let reader: Box<dyn BufRead> = if !input.is_empty() {
    //     Box::new(BufReader::new(File::open(input).unwrap()))
    // } else {
    //     Box::new(BufReader::new(tokio::io::stdin()))
    // };

    let request_worker = tokio::spawn(async move {
        for url_result in reader.lines() {
            let request_context = RequestContext {
                url: url_result.unwrap(),
            };

            let resp = task::spawn(request(request_context, client.clone()));

            if request_tx.send(resp).await.is_err() {
                println!("can't transmit");
                return;
            }
        }
    });

    let response_awaiter = tokio::spawn(async move {
        let mut bu = request_rx.buffer_unordered(20);

        while let Some(_handle) = bu.next().await {}
    });

    let _ = request_worker.await;
    let _ = response_awaiter.await;

    Ok(())
}

fn main() -> Result<()> {
    let future = app();
    let mut rt = runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()?;

    rt.block_on(future)
}

async fn request(request_context: RequestContext, client: Client) -> Result<()> {
    let start = std::time::Instant::now();
    let url = Url::parse(&request_context.url).unwrap();
    let response = client.get(url.clone()).send().await?;

    // dummy latency on some subset of requests
    // if id % 2 == 0 {
    //     delay_for(Duration::from_millis(100)).await;
    // }

    let elapsed = start.elapsed().as_millis();

    // TODO switch output to a byte stream? something more configurable than just text?
    println!(
        "{} : {}ms -> {}",
        response.status(),
        elapsed,
        response.text().await?
    );

    Ok(())
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
