use crate::utils::astronauts_url;
use crate::utils::auth_url;
use crate::utils::current_timestamp;
use crate::utils::missions_url;
use futures::future::join_all;
use futures::StreamExt;
use hyper::Body;
use hyper::Client;
use hyper::Request;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize, Serialize)]
struct TokenPair {
    access_token: String,
}

#[tokio::test]
async fn missions_should_be_created() {
    const ITERATIONS: i32 = 10;
    const OPERATIONS: i32 = 100;
    const SLEEP_MS: u64 = 2000;

    let client = Client::new();
    let astronaut_name = format!("astro_{}_0000", current_timestamp());

    let req_1 = Request::post(astronauts_url("/astronauts"))
        .header("Content-Type", "application/json")
        .body(Body::from(format!(
            r#"{{
                    "name": "{}",
                    "password": "1234",
                    "birth_date": "1994-06-25T00:00:00Z"
                }}"#,
            astronaut_name
        )))
        .expect("issue creating request");
    let res_1 = client.request(req_1).await.expect("error in response");
    assert_eq!(res_1.status(), 201);

    sleep(Duration::from_millis(1000)).await;

    let req_2 = Request::post(auth_url("/token_pair"))
        .header("Content-Type", "application/json")
        .body(Body::from(format!(
            r#"{{
                    "name": "{}",
                    "password": "1234"
                }}"#,
            astronaut_name
        )))
        .expect("issue creating request");
    let mut res_2 = client.request(req_2).await.expect("error in response");
    assert_eq!(res_2.status(), 200);
    let body = res_2.body_mut().next().await.unwrap().unwrap();
    let body = serde_json::from_slice::<TokenPair>(&body).unwrap();
    let access_token = &body.access_token;

    for _ in 0..ITERATIONS {
        let futures = (0..OPERATIONS).map(|i| async move {
            let client = Client::new();
            let mission_name = format!("mission_{}_{}", current_timestamp(), i);

            let req_3 = Request::post(missions_url("/missions"))
                .header("Content-Type", "application/json")
                .header("Authorization", &format!("Bearer {}", &access_token))
                .body(Body::from(format!(
                    r#"{{
                    "name": "{}",
                    "start_date": "1994-06-25T00:00:00Z"
                }}"#,
                    mission_name
                )))
                .expect("issue creating request");
            let res_3 = client.request(req_3).await.expect("error in response");
            assert_eq!(res_3.status(), 201);

            sleep(Duration::from_millis(SLEEP_MS)).await;
        });

        join_all(futures).await;
    }
}
