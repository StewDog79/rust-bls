use std::error::Error;
use std::io::Read;
use flate2::read::GzDecoder;
use bytes::buf::BufExt;
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT_LANGUAGE, ACCEPT, ACCEPT_ENCODING, CONNECTION, REFERER};
use select::document::Document;
use select::predicate::Name;
use tokio::runtime::Runtime;

async fn get_ppi_data(url: &str) -> Result<String, Box<dyn Error>> {
    // Creating the headers
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client
        .get(url)
        .header(ACCEPT_ENCODING, "gzip") // Request gzipped content
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Error while fetching the data: {}", response.status()).into());
    }

    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE);
    let mut body = Vec::new();

    if let Some(content_type) = content_type {
        if content_type.to_str().unwrap().contains("gzip") {
            let bytes = response.bytes().await?;
            let mut decoder = GzDecoder::new(bytes.reader());
            decoder.read_to_end(&mut body)?;
        } else {
            response.read_to_end(&mut body)?;
        }
    } else {
        response.read_to_end(&mut body)?;
    }

    let html_content = String::from_utf8(body)?;
    Ok(html_content)
}

fn main() {
    let url = "https://www.bls.gov/news.release/ppi.nr0.htm";

    // Build and run the tokio runtime to execute the async main function
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        match get_ppi_data(url).await {
            Ok(html_content) => {
                // Parse the HTML content using select crate
                let document = Document::from(html_content.as_str());

                // Extract the text from all <p> tags and print them
                for node in document.find(Name("p")) {
                    let text = node.text();
                    if !text.trim().is_empty() {
                        println!("{}", text.trim());
                    }
                }
            }
            Err(err) => println!("Error while fetching the data: {}", err),
        }
    });
}
