use crate::lib;

fn load_env() {
    let _ = dotenvy::dotenv();
}

#[tokio::test]
async fn test_invalid_token() {
    unsafe{
        std::env::set_var("HARDCOVER_API_TOKEN", "invalid");
        std::env::set_var(
            "HARDCOVER_API_ENDPOINT",
            "https://api.hardcover.app/v1/graphql"
        );
    }


    let result = lib::info_sender::get_series_title("Behind the Throne","K. B. Wagers",).await;

    assert!(result.is_err());
}


#[tokio::test]
async fn test_invalid_url() {
    unsafe{
        std::env::set_var("HARDCOVER_API_TOKEN", "invalid");
        std::env::set_var(
            "HARDCOVER_API_ENDPOINT",
            "bla"
        );
    }


    let result = lib::info_sender::get_series_title("Behind the Throne","K. B. Wagers",).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_valid_token_and_url() {
    load_env();
    let result = lib::info_sender::get_series_title("Behind the Throne","K. B. Wagers",).await;
    println!("{:?}",result);
    assert!(result.is_ok());
}