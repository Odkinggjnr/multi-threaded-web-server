use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use web_server::ThreadPool;

fn main() {
    // start listening on port 7878 (the front door of our lemonade stand)
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    println!("Server running on http://127.0.0.1:7878");

    // every time someone connects, hand the work to a worker thread
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

// reads what the browser asked for, then sends back the right page
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");

    // pick the right file based on what was requested
    let (status_line, filename, content_type) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html", "text/html")
    } else if request_line == "GET /hello.css HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.css", "text/css")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    // build and send the HTTP response
    let response = format!(
        "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}
