extern crate hyper;
extern crate hyper_native_tls;

use std::{process, str};
// use hyper::client::response::Response;
use hyper::Client;
use hyper::client::Response;
use hyper::net::HttpsConnector;
use hyper::header::{Connection, Headers, UserAgent};
use hyper_native_tls::NativeTlsClient;
use std::io::Read;
use std::io::Write;
use std::string::String;
use std::collections::HashMap;
use std::fs::File;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let parsed_url = hyper::Url::parse(&args[1]).unwrap();
        let video_id: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
        let url = format!("https://youtube.com/get_video_info?video_id={}",
                          video_id["v"]);
        download(&url);
    } else {
        println!("Specify youtube url");
    }
}

fn download(url: &str) {
    let mut response = send_request(url);
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();
    let hq = parse_url(&response_str);

    if hq["status"] != "ok" {
        println!("Video not found");
        process::exit(1);
    }

    let streams: Vec<&str> = hq["url_encoded_fmt_stream_map"].split(',').collect();
    let mut qualities: HashMap<i32, (String, String)> = HashMap::new();
    for (i, url) in streams.iter().enumerate() {
        let quality = parse_url(url);
        let extension = quality["type"]
            .split('/')
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        qualities.insert(i as i32, (quality["url"].to_string(), extension.to_owned()));
        println!("{}- {} {}", i, quality["quality"], quality["type"]);
    }

    println!("Choose quality (0):");
    let input = read_line().trim().parse().unwrap_or(0);

    println!("Please wait ...");

    let url = &qualities[&input].0;
    let extension = &qualities[&input].1;

    let response = send_request(url);
    println!("{}", url);
    println!("Download is starting ...");

    let filename = format!("{}.{}", hq["title"], extension);

    write_file(response, &filename);
}

fn send_request(url: &str) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client
        .get(url)
        .header(Connection::close())
        .header(UserAgent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/57.0.2987.133 Safari/537.36".to_string()))
        .send()
        .unwrap_or_else(|e| {
                            println!("Network request failed: {}", e);
                            process::exit(1);
                        })
}

fn write_file(mut response: Response, title: &str) {
    let mut file = File::create(title).unwrap();
    let mut body: Vec<u8> = vec![];
    response.read_to_end(&mut body).unwrap();
    file.write_all(&body);
}

fn parse_url(query: &str) -> HashMap<String, String> {
    let u = format!("{}{}", "https://e.com?", query);
    let parsed_url = hyper::Url::parse(&u).unwrap();
    parsed_url.query_pairs().into_owned().collect()
}

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Could not read stdin");
    input
}
