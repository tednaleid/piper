use futures::StreamExt;
use reqwest::{Client, Url};
use std::io::{self, BufRead, BufReader};
use tokio::sync::mpsc::{self, Receiver};
use tokio::{runtime, task};
// use tokio::time::{delay_for, Duration};
use piper::args::Args;
use std::fs::File;
use tokio::time::Duration;
use std::alloc::handle_alloc_error;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    let Args { input } = Args::parse();

    let request_client = request_client()?;

    let (mut request_context_tx, mut request_context_rx) = mpsc::channel(16);

    // plays into the max number of concurrent requests (one being sent, N in the channel, and M in the buffer_unordered on the other end)
    // the unordered buffer in the response awaiter also plays into it
    let (mut request_tx, request_rx) = mpsc::channel(16);

    let response_awaiter = tokio::spawn(async move {
        let mut bu = request_rx.buffer_unordered(20);

        while let Some(handle) = bu.next().await {
            let result = match handle {
                Ok(_) => { },
                Err(e) => {
                    eprintln!("error! {}", e);
                },
            };
        }
    });

    let request_maker = tokio::spawn(async move {
        while let Some(request_context) = request_context_rx.recv().await {
            let resp = task::spawn(request(request_context, request_client.clone()));
            if request_tx.send(resp).await.is_err() {
                println!("can't transmit");
                break;
            }
        };
    });

    // let reader: Box<BufReader<Stdin>> = Box::new(BufReader::new(io::stdin()));
    let reader = create_reader(input)?;

    for url_result in reader.lines() {
        let request_context = RequestContext {
            url: url_result.unwrap(),
        };

        if request_context_tx.send(request_context).await.is_err() {
            eprintln!("can't transmit");
            break
        }
    }

    // need to explicitly drop it so it closes and we can finish
    drop(request_context_tx);

    let _ = request_maker.await;
    let _ = response_awaiter.await;

    Ok(())
}

fn create_reader(input: String) -> Result<Box<dyn BufRead>> {
    let reader: Box<dyn BufRead> = if !input.is_empty() {
        Box::new(BufReader::new(File::open(input)?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };
    Ok(reader)
}

fn request_client() -> Result<Client> {
    let timeout = Duration::new(10, 0);
    let client = Client::builder().timeout(timeout).gzip(true).build()?;
    Ok(client)
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
