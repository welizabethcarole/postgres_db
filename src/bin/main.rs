extern crate postgres;

use postgres::{Client, NoTls, Error};
use std::collections::HashMap;

use hello::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

struct Author {
    _id: i32,
    name: String,
    country: String
}


fn main() -> Result<(), Error> {
    let mut client = Client::connect("postgresql://postgres:NO6ekYSmkv4kSG9wOLUieSFPw@50.116.20.126:48278/test", NoTls)?;
    
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS author (
            id              SERIAL PRIMARY KEY,
            name            VARCHAR NOT NULL,
            country         VARCHAR NOT NULL
            )
    ")?;

    client.batch_execute("
        CREATE TABLE IF NOT EXISTS book  (
            id              SERIAL PRIMARY KEY,
            title           VARCHAR NOT NULL,
            author_id       INTEGER NOT NULL REFERENCES author
            )
    ")?;

    let mut authors = HashMap::new();
    authors.insert(String::from("Chinua Achebe"), "Nigeria");
    authors.insert(String::from("Rabindranath Tagore"), "India");
    authors.insert(String::from("Anita Nair"), "India");

    for (key, value) in &authors {
        let author = Author {
            _id: 0,
            name: key.to_string(),
            country: value.to_string()
        };

        client.execute(
                "INSERT INTO author (name, country) VALUES ($1, $2)",
                &[&author.name, &author.country],
        )?;
    }

    for row in client.query("SELECT id, name, country FROM author", &[])? {
        let author = Author {
            _id: row.get(0),
            name: row.get(1),
            country: row.get(2),
        };
        println!("Author {} is from {}", author.name, author.country);
    }

    // CASCADE allows you to remove the table and dependent objects
    // RESTRICT rejects the removal if there is any object that depends on the table.
    client.batch_execute(
        "
        DROP TABLE IF EXISTS author CASCADE
        "
    )?;

    client.batch_execute(
        "
        DROP TABLE IF EXISTS book RESTRICT
        "
    )?;

    // create database
    /*client.batch_execute(
        "
        CREATE DATABASE myNewDB
        "
    )?;

    // delete database
    client.batch_execute(
        "
        DROP DATABASE myNewDB
        "
    )?;*/

    /* listening for TCP connections at 127.00.1:7878
       7878 is Rust typed out on a telephone. */
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    /* threadpool interface
       We use ThreadPool::new to create a new thread pool with a configurable number of threads(workers) */
    let pool = ThreadPool::new(4);
   
       /* shut down process after xx requests */
    for stream in listener.incoming().take(4) {
        let stream = stream.unwrap();
   
        pool.execute(|| {
            handle_connection(stream);
        });
    }
   
    println!("Shutting down.");


    Ok(())

}


fn handle_connection(mut stream: TcpStream) {
    /* buffer holds the data that is being read in */
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    /* hardcode the data corresponding to the / request into the get variable */
    let get = b"GET / HTTP/1.1\r\n";

    /* for simulating a slow request */
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    
    /* 127.0.0.1:7878 */
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } 
    /* 127.0.0.1:7878/sleep; waits 5 seconds to respond */
    else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    }
    /* else if there is 127.0.0.1:7878/someotherpath that is not recognized, gives 404 error */
    else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    if buffer.starts_with(get) {
        /* rendering html on the server */
        let contents = fs::read_to_string(filename).unwrap();

        /* standard success response */
        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );
        /* sends the response as bytes directly to the connection; unwrap is used for error messages */
        stream.write(response.as_bytes()).unwrap();

        /* flush will wait and prevent the program from continuing until all the bytes are written to the connection */
        stream.flush().unwrap();
    } else {
        /* error page, loads 404.html */
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap();


        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}