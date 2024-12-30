use std::str;
use serde_json::{json, Value};
use tiny_http::Response;

fn main() {
    let server = tiny_http::Server::http("0.0.0.0:8080").unwrap();
    loop {
        let mut request = match server.recv() {
            Ok(rq) => rq,
            Err(e) => { eprintln!("Request failed with error: {:?}", e); break }
        };
        println!("Recieved request! Method: {:?}, Url: {:?}",
            request.method(),
            request.url()
        );
        
        let mut request_body = String::new();
        request.as_reader().read_to_string(&mut request_body).unwrap();
        let request_body: Value = serde_json::from_str(request_body.as_str()).unwrap();

        println!("{} says: {}",
            request_body
                .get("message")
                .and_then(|m| m.get("from"))
                .and_then(|f| f.get("username"))
                .and_then(|username| username.as_str())
                .unwrap(),
            request_body
                .get("message")
                .and_then(|m| m.get("text"))
                .and_then(|text| text.as_str())
                .unwrap()
        );

        let response_message = "Message has been recieved!".to_string();
        let response = Response::from_string(response_message).with_status_code(200);
        
        if let Err(e) = request.respond(response) {
            eprint!("Response failed with error: {:?}", e);
        }

        send_message(&request_body,
             match request_body
                .get("message")
                .and_then(|m| m.get("text"))
                .and_then(|text| text.as_str()) {
                    Some("ping") => "pong",
                    Some(_) => "I do not know what u just said!!!",
                    None => "message not recieved",
            }
        )
    }
}

fn send_message(request_body: &Value, text: &str) {
    if let Some(chat_id) = request_body
    .get("message")
    .and_then(|m| m.get("chat"))
    .and_then(|c| c.get("id"))
    .and_then(|id| id.as_i64()) {
        let telegram_message = json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown"
        });

        

        match minreq::post(format!("http://api.telegram.org/bot{:?}/sendMessage", std::env::var("API_TOKEN")))
            .with_header("Content-Type", "application/json")
            .with_body(telegram_message.to_string())
            .send()
        {
            Ok(response) => println!("Replied to {} with: {}",
                request_body
                    .get("message")
                    .and_then(|m| m.get("from"))
                    .and_then(|f| f.get("username"))
                    .and_then(|username| username.as_str())
                    .unwrap(),
                serde_json::from_str::<Value>(response.as_str().unwrap())
                    .unwrap()
                    .get("result")
                    .and_then(|r| r.get("text"))
                    .unwrap()
                    .as_str()
                    .unwrap()
            ),
            Err(e) => eprintln!("Error sending Telegram message: {}", e),
        }
    } else {
        eprintln!("Invalid chat_id in request body.");
    }
}