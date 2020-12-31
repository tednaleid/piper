use anyhow::{Error, Result};
use futures::{Future, StreamExt};
use piper::args::Args;
use piper::context::{FieldValues, OutputTemplate, SPACE_BYTE};
use reqwest::{Client, Method, Url, StatusCode};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tokio::runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::{delay_for, Duration};

pub async fn app() -> Result<()> {
    let Args {
        input,
        method,
        url,
        parallel,
        timeout_seconds,
    } = Args::parse()?;

    let request_client = request_client(timeout_seconds)?;

    let (mut request_context_tx, mut request_context_rx) = mpsc::channel(256);

    let (mut request_tx, request_rx) = mpsc::channel(256);

    let (mut response_tx, mut response_rx) = mpsc::channel(256);

    // TODO punting on refactoring this till reqwest is updated to support tokio 1.0: https://github.com/seanmonstar/reqwest/issues/1123
    // TODO these could be transformed into something closer to an actor and paired with the thing
    // doing the receiving where they have a mailbox that we can get access to (the tx) and the rx is internal
    let response_awaiter = tokio::spawn(async move {
        let mut bu = request_rx.buffer_unordered(parallel);

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
            // might be nice to have this as a task as then we could do more in that task, such as retries/following redirects/etc
            // let resp = task::spawn(request(request_context, request_client.clone()));
            let resp = request(request_context, request_client.clone(), response_tx.clone());
            if request_tx.send(resp).await.is_err() {
                eprintln!("can't transmit");
                break;
            }
        }
    });

    let output_handler = tokio::spawn(async move {
        while let Some(response_context) = response_rx.recv().await {
            println!("{}",  response_context.text);
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
            method: method.clone(),
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
    let _ = output_handler.await;

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

fn request_client(timeout_seconds: u64) -> Result<Client> {
    let timeout = Duration::new(timeout_seconds, 0);
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

async fn request(request_context: RequestContext, client: Client, mut response_tx: Sender<ResponseContext>) -> Result<()> {
    let start = std::time::Instant::now();
    let url = Url::parse(&request_context.url).unwrap();
    let response = client
        .request(request_context.method.clone(), url.clone())
        .send()
        .await?;


    // dummy latency on some subset of requests
    // if request_context.id % 1 == 0 {
    //     delay_for(Duration::from_millis(1000)).await;
    // }

    let response_context = ResponseContext {
        request_context,
        status: response.status(),
        text: response.text().await?,
        elapsed: start.elapsed(),
    };

    response_tx.send(response_context).await?;

    Ok(())
}

#[derive(Debug)]
pub struct RequestContext {
    url: String,
    method: Method,
    id: i64,
}

#[derive(Debug)]
pub struct ResponseContext {
    request_context: RequestContext,
    status: StatusCode,
    text: String,
    // eventually this will be bytes or something else
    elapsed: Duration,
}


impl PartialEq for RequestContext {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.method == other.method && self.id == other.id
    }
}
