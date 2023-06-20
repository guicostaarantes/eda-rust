// The idea of this file is to demonstrate how to test with the hyper client,
// including SSE (long-lived HTTP requests).

use crate::utils::astronauts_url;
use futures::task::Poll;
use futures::StreamExt;
use futures_test::task::noop_context;
use hyper::body::Bytes;
use hyper::Body;
use hyper::Client;
use hyper::Request;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn time_elapsed() {
    let client = Client::new();
    let req = Request::get(astronauts_url("/time-elapsed"))
        .body(Body::from(""))
        .expect("issue creating request");
    let mut response = client.request(req).await.expect("error in response");
    let stream = response.body_mut();

    // Easier way to test is awaiting the next message from the stream
    for i in 0..=5 {
        assert!(match stream.next().await {
            Some(Ok(bytes)) => {
                &bytes[..] == Bytes::from(format!("event:time_elapsed\ndata:{}\n\n", i))
            }
            _ => false,
        });
    }

    // If it's required to test if the stream is in a pending state, use poll_next_unpin
    // It requires a context, so pass a noop_context to it
    let mut cx = noop_context();
    // We are expecting a new message in 1000 ms, so sleeping for just 500ms and checking
    // the stream state should return Poll::Pending
    sleep(Duration::from_millis(500)).await;
    assert!(match stream.poll_next_unpin(&mut cx) {
        Poll::Pending => true,
        _ => false,
    });

    // It's also possible to test for ready state using poll_next_unpin, but in this example
    // we had to sleep the thread to allow time for the stream to deliver, whereas with
    // stream.next().await it automatically hangs until it's ready
    for i in 6..=10 {
        sleep(Duration::from_millis(1000)).await;
        assert!(match stream.poll_next_unpin(&mut cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                &bytes[..] == Bytes::from(format!("event:time_elapsed\ndata:{}\n\n", i))
            }
            _ => false,
        });
    }

    // Use either next().await or poll_next_unpin to check for the end of the stream
    sleep(Duration::from_millis(1000)).await;
    assert!(match stream.poll_next_unpin(&mut cx) {
        Poll::Ready(None) => true,
        _ => false,
    });
    assert!(match stream.next().await {
        None => true,
        _ => false,
    })
}
