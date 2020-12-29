use anyhow::{Error, Result};
use futures::{Future, StreamExt};
use piper::args::Args;
use piper::context::{FieldValues, OutputTemplate, SPACE_BYTE};
use reqwest::{Client, Method, Url};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tokio::runtime;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;
use tokio::time::Duration;

pub async fn app() -> Result<()> {
    let Args { input, method, url } = Args::parse();

    let request_client = request_client()?;

    let (mut request_context_tx, mut request_context_rx) = mpsc::channel(256);

    let (mut request_tx, request_rx) = mpsc::channel(256);

    // TODO punting on refactoring this till reqwest is updated to support tokio 1.0: https://github.com/seanmonstar/reqwest/issues/1123

    let response_awaiter = tokio::spawn(async move {
        // number of concurrent requests that we're awaiting
        // TODO make this a config value
        let mut bu = request_rx.buffer_unordered(16);

        // how should retries be handled? one option, if it isn't built into reqwest, would be to send the request_context back down the pipe with a counter
        // looks like reqwest has a try_clone on the request object for retry reasons (the try is because if the body is a stream, it can't clone that)

        while let Some(handle) = bu.next().await {
            if let Err(e) = handle {
                eprintln!("error! {}", e);
            };
        }
    });

    let request_maker = tokio::spawn(async move {
        while let Some(request_context) = request_context_rx.recv().await {
            // TODO decide about this, spawning a task gives about 2x the throughput, but adds a level of indirection on the awaiting
            // let resp = task::spawn(request(request_context, request_client.clone()));
            let resp = request(request_context, request_client.clone());
            if request_tx.send(resp).await.is_err() {
                eprintln!("can't transmit");
                break;
            }
        }
    });

    let reader = create_reader(input)?;

    let field_separator: u8 = SPACE_BYTE;

    let url_template = OutputTemplate::parse(url.as_str());

    let mut line_count = 1;

    for line_result in reader.lines() {
        let line = line_result?;
        let values = FieldValues::parse(line.as_bytes(), field_separator, 1);

        let url = url_template.merge(values)?;

        let request_context = RequestContext {
            url,
            method: Method::GET,
            id: line_count,
        };

        line_count += 1;

        if request_context_tx.send(request_context).await.is_err() {
            eprintln!("can't transmit");
            break;
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
    let response = client
        .request(request_context.method, url.clone())
        .send()
        .await?;

    // dummy latency on some subset of requests
    // if request_context.id % 2 == 0 {
    //     delay_for(Duration::from_millis(1000)).await;
    // }

    let elapsed = start.elapsed().as_millis();

    // TODO switch output to a byte stream? something more configurable than just text?
    println!(
        "{} : {}ms -> {} - {:?}",
        response.status(),
        elapsed,
        response.text().await?,
        std::thread::current().id()
    );

    Ok(())
}

#[derive(Debug)]
pub struct RequestContext {
    url: String,
    method: Method,
    id: i64,
}

impl PartialEq for RequestContext {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.method == other.method && self.id == other.id
    }
}
