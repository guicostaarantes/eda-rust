use crate::utils::astronauts_url;
use crate::utils::auth_url;
use crate::utils::current_timestamp;
use futures::future::join_all;
use hyper::Body;
use hyper::Client;
use hyper::Request;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn create_astronaut_should_422_on_bad_input() {
    let client = Client::new();

    let req = Request::post(astronauts_url("/astronauts"))
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{
                "name": "gui",
                "password": "1234",
                "birth_date": "not really a ISO 8601 date string"
            }"#,
        ))
        .expect("issue creating request");
    let res = client.request(req).await.expect("error in response");
    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn astronaut_should_be_fetched_2000ms_after_creation() {
    const ITERATIONS: i32 = 10;
    const OPERATIONS: i32 = 500;
    const SLEEP_MS: u64 = 2000;

    for j in 0..ITERATIONS {
        let timestamp = current_timestamp();
        println!(
            "creating {} astronauts - iter {} at {}",
            OPERATIONS,
            j + 1,
            chrono::Local::now()
        );

        let futures = (0..OPERATIONS).map(|i| async move {
            let client = Client::new();
            let astronaut_name = format!("astro_{}_{}", &timestamp, format!("{:0>4}", i));

            let req = Request::post(astronauts_url("/astronauts"))
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "name": "{astronaut_name}",
                        "password": "1234",
                        "birth_date": "1994-06-25T00:00:00Z"
                    }}"#,
                )))
                .expect("issue creating request");
            let res = client.request(req).await.expect("error in response");
            assert_eq!(res.status(), 201);
        });

        join_all(futures).await;
        println!("finished at {}", chrono::Local::now());
        sleep(Duration::from_millis(SLEEP_MS)).await;
        println!(
            "authenticating {} astronauts - iter {} at {}",
            OPERATIONS,
            j + 1,
            chrono::Local::now()
        );

        let futures = (0..OPERATIONS).map(|i| async move {
            let client = Client::new();
            let astronaut_name = format!("astro_{}_{}", &timestamp, format!("{:0>4}", i));

            let req = Request::post(auth_url("/token_pair"))
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "name": "{astronaut_name}",
                        "password": "1234"
                    }}"#,
                )))
                .expect("issue creating request");
            let res = client.request(req).await.expect("error in response");
            assert_eq!(res.status(), 200);
        });

        join_all(futures).await;
        println!("finished at {}", chrono::Local::now());
    }
}
