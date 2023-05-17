use eventsource_client::Client;
use eventsource_client::ClientBuilder;
use eventsource_client::Error;
use eventsource_client::ReconnectOptions;
use eventsource_client::Result;
use eventsource_client::SSE;
use futures::TryStreamExt;

#[derive(Debug, Default)]
struct Args {
    url: String,
    token: String,
    body: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::default();
    let mut args_iter = std::env::args();
    while let Some(arg) = args_iter.next() {
        if arg == "--url" {
            args.url = args_iter.next().expect("no value for --url");
        }
        if arg == "--token" {
            args.token = args_iter.next().expect("no value for --url");
        }
        if arg == "--body" {
            args.body = args_iter.next().expect("no value for --body");
        }
    }
    println!("{:?}", args);

    let client = ClientBuilder::for_url(&args.url)?
        .header("Content-Type", "application/json")?
        .header("Authorization", &format!("Bearer {}", args.token))?
        .body(args.body.clone())
        .reconnect(ReconnectOptions::reconnect(false).build())
        .build();

    let mut stream = client
        .stream()
        .map_ok(|e| match e {
            SSE::Event(event) => {
                println!("event: {}\ndata: {}\n", event.event_type, event.data)
            }
            SSE::Comment(comment) => {
                println!("comment: \n{}\n", comment)
            }
        })
        .map_err(|err| {
            match err {
                Error::Eof => println!("end of stream"),
                _ => eprintln!("error: {:?}", err),
            };
        });

    while let Ok(Some(_)) = stream.try_next().await {}

    Ok(())
}
