use futures::StreamExt;
use reqwest::{Client, Url};
use std::io::{self, BufRead, BufReader};
use tokio::sync::mpsc::{self, Receiver};
use tokio::{runtime, task};
// use tokio::time::{delay_for, Duration};
use piper::args::Args;
use std::fs::File;
use tokio::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    let Args { input } = Args::parse();

    let request_client = request_client()?;

    // plays into the max number of concurrent requests (one being sent, N in the channel, and M in the buffer_unordered on the other end)
    // the unordered buffer in the response awaiter also plays into it
    let (mut request_tx, request_rx) = mpsc::channel(16);

    let response_awaiter = tokio::spawn(async move {
        let mut bu = request_rx.buffer_unordered(20);

        while let Some(_handle) = bu.next().await {}
    });

    // let reader: Box<BufReader<Stdin>> = Box::new(BufReader::new(io::stdin()));
    let reader = create_reader(input)?;

    for url_result in reader.lines() {
        let request_context = RequestContext {
            url: url_result.unwrap(),
        };

        let resp = task::spawn(request(request_context, request_client.clone()));

        if request_tx.send(resp).await.is_err() {
            println!("can't transmit");
            break;
        }
    }

    drop(request_tx);

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

// pub fn write_loop(outfile: &str, write_rx: Receiver<Vec<u8>>) -> Result<()> {
//     let mut writer: Box<dyn Write> = if !outfile.is_empty() {
//         Box::new(BufWriter::new(File::create(outfile)?))
//     } else {
//         Box::new(BufWriter::new(io::stdout()))
//     };
//
//     loop {
//         let buffer = write_rx.recv().unwrap();
//
//         if buffer.is_empty() {
//             break;
//         }
//
//         if let Err(e) = writer.write_all(&buffer) {
//             if e.kind() == ErrorKind::BrokenPipe {
//                 return Ok(());
//             }
//             return Err(e);
//         }
//     }
//     Ok(())
// }

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
