extern crate hyper;
extern crate hyper_native_tls;

use std::{process, str};
// use hyper::client::response::Response;
use hyper::Client;
use hyper::client::Response;
use hyper::net::HttpsConnector;
use hyper::header::ContentLength;
use hyper_native_tls::NativeTlsClient;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;
use std::fs::File;

fn main() {
    let url = "https://youtube.com/get_video_info?video_id=l6zpi90IT1g";
    download(&url);
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
    println!("Download is starting ...");

    let file_size = get_file_size(&response);
    let filename = format!("{}.{}", hq["title"], extension);

    write_file(response, &filename, file_size);
}

fn send_request(url: &str) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client
        .get(url)
        .send()
        .unwrap_or_else(|e| {
                            println!("Newwork request failed: {}", e);
                            process::exit(1);
                        })
}

fn write_file(mut response: Response, title: &str, file_size: u64) {
    let mut buf = [0; 128 * 1024];
    let mut file = File::create(title).unwrap();
    loop {
        match response.read(&mut buf) {
            Ok(len) => {
                file.write_all(&buf[..len]).unwrap();
                len
            }
            Err(why) => panic!("{}", why),
        };
    }
}

fn get_file_size(response: &Response) -> u64 {
    let mut file_size = 0;
    match response.headers.get::<ContentLength>() {
        Some(length) => file_size = length.0,
        None => println!("Content-length header missing"),
    };
    file_size
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
