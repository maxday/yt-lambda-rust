use lambda_http::{
    aws_lambda_events::serde_json::json, run, service_fn, Body, Error, IntoResponse, Request,
    RequestExt, Response,
};
use serde::Serialize;

#[derive(Serialize)]
struct Pizza {
    name: String,
    price: i32,
}

struct PizzaList {
    pizzas: Vec<Pizza>,
}

impl PizzaList {
    fn new() -> PizzaList {
        let veggie = Pizza {
            name: String::from("veggie"),
            price: 10,
        };
        let regina = Pizza {
            name: String::from("regina"),
            price: 12,
        };
        let deluxe = Pizza {
            name: String::from("deluxe"),
            price: 14,
        };
        PizzaList {
            pizzas: vec![veggie, regina, deluxe],
        }
    }
}

fn build_error(error_message: &str) -> Response<Body> {
    Response::builder()
        .status(400)
        .header("content-type", "application/json")
        .body(lambda_http::Body::from(
            json!({ "error": error_message }).to_string(),
        ))
        .expect("impossible to build the error response")
}

async fn process_event(event: Request, pizza_list: &PizzaList) -> Response<Body> {
    match event.path_parameters().first("pizza_name") {
        Some(pizza_name) => {
            let mut iter = pizza_list.pizzas.iter();
            if let Some(found_pizza) = iter.find(|&pizza| pizza.name == pizza_name) {
                json!(found_pizza).into_response().await
            } else {
                build_error("no pizza found for the given pizza_name")
            }
        }
        _ => build_error("could not find the pizza_name"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let all_pizza_ref = &PizzaList::new();
    let handler_func_closure = |event: Request| async move {
        Result::<Response<Body>, Error>::Ok(process_event(event, all_pizza_ref).await)
    };
    run(service_fn(handler_func_closure)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use maplit::hashmap;

    #[test]
    fn build_error_test() {
        let result = build_error("test error message");
        let (parts, body) = result.into_parts();
        let content_type_value = parts.headers.get("content-type").unwrap();
        let status_code = parts.status;
        let ut8_body = String::from_utf8(body.to_ascii_lowercase()).unwrap();
        assert_eq!(ut8_body, "{\"error\":\"test error message\"}");
        assert_eq!(content_type_value, "application/json");
        assert_eq!(status_code, 400);
    }

    #[tokio::test]
    async fn process_event_valid_pizza() {
        let mocked = hashmap! {
            "pizza_name".into() => vec!["regina".into()]
        };
        let pizza_list = PizzaList::new();
        let event = Request::default().with_path_parameters(mocked.clone());
        let response = process_event(event, &pizza_list);
        let into_response = response.await;
        assert_eq!(into_response.status(), 200);
    }

    #[tokio::test]
    async fn process_event_unknown_pizza() {
        let mocked = hashmap! {
            "other_invalid_param".into() => vec!["test_value".into()]
        };
        let pizza_list = PizzaList::new();
        let event = Request::default().with_path_parameters(mocked.clone());
        let response = process_event(event, &pizza_list);
        let into_response = response.await;
        assert_eq!(into_response.status(), 400);

        let ascii_body = into_response.into_body().to_ascii_lowercase();
        let ut8_body = String::from_utf8(ascii_body).unwrap();
        assert_eq!(ut8_body, "{\"error\":\"could not find the pizza_name\"}")
    }

    #[tokio::test]
    async fn handler_pizza_name_incorrect_test() {
        let mocked = hashmap! {
            "pizza_name".into() => vec!["not_a_valid_pizza_name".into()]
        };
        let pizza_list = PizzaList::new();
        let event = Request::default().with_path_parameters(mocked.clone());
        let response = process_event(event, &pizza_list);
        let into_response = response.await;
        assert_eq!(into_response.status(), 400);

        let ascii_body = into_response.into_body().to_ascii_lowercase();
        let ut8_body = String::from_utf8(ascii_body).unwrap();
        assert_eq!(
            ut8_body,
            "{\"error\":\"no pizza found for the given pizza_name\"}"
        )
    }
}
