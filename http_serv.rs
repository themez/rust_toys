// Running on Rust 0.11

#[feature(globs)];
use std::io::*;
use std::io::fs::Directories;
use std::io::net::ip::{SocketAddr};
use std::{str};
use std::os;
use std::fmt;

static IP: &'static str = "127.0.0.1";
static PORT:        int = 4414;

fn main() {
    let addr = from_str::<SocketAddr>(format!("{:s}:{:d}", IP, PORT)).unwrap();
    let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
    println(format!("Listening on [{:s}] ...", addr.to_str()));
    

    fn not_found(mut stream:IoResult<TcpStream>){
        let res = ~"HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
            <doctype !html><html><head><title>404</title><body>File not found!</body>";
        stream.write(res.as_bytes());
    }

    fn forbidden(mut stream:IoResult<TcpStream>){
       let res = ~"HTTP/1.1 403 Forbidden\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
            <doctype !html><html><head><title>404</title><body>Not authorized!</body>";
        stream.write(res.as_bytes()); 
    }

    fn serv(mut stream:IoResult<TcpStream>, file_path:&Path){
        let resource_file = File::open(file_path);//File::open(&Path::new(file_path));
        match resource_file {
            Ok(mut res_byte) => {
                match res_byte.read_to_end() {
                    Ok(bytes) => {
                        stream.write("HTTP/1.1 200\r\ncharset=UTF-8\r\n\r\n".as_bytes());
                        stream.write(bytes.as_slice());
                    },
                    _=>{not_found(stream);}
                }
            },
            _ =>{
                not_found(stream);
            }
        }
    }

    fn dir(mut stream:IoResult<TcpStream>, file_path:&Path){
        println!("list files:{}", file_path.display());
        let mut res = ~[~"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                        <html><head><title>folder</title><body><ul>"];
        res.push(~"<li><a href=\"..\">..</a></li>");
        match fs::readdir(file_path){
            Ok(paths)=>{
                for file in paths.iter() {
                    
                    match file.filename() {
                        Some(name)=>{
                            match str::from_utf8(name) {
                                Some(name_str)=>{
                                    
                                    if file.is_dir(){
                                        let mut li:~str = ~"<li><a href=\""+name_str+"/\">"+name_str+~"</a></li>";
                                        res.push(li);
                                    }else{
                                        let mut li:~str = ~"<li><a href=\""+name_str+"\">"+name_str+~"</a></li>";
                                        res.push(li);
                                    }
                                },
                                None=>()
                            }
                        },
                        None=>()
                    }
                }
            },
            _=>()
        };
        res.push(~"</ul></body>");
        for r in res.iter() {
            stream.write(r.as_bytes());
        }
    }

    for stream in acceptor.incoming() {

        spawn (proc(){

            let mut stream = stream;
            
            match stream {
                Ok(ref mut s) => {
                    match s.peer_name() {
                        Ok(pn) => {println(format!("Received connection from: [{:s}]", pn.to_str()));},
                        _ => ()
                    }
                },
                _ => ()
            }
            
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            let cwd = os::getcwd();

            match request_str {
                Some(req_str) =>{
                    println(format!("Received request :\n{:s}", req_str));
                    //serv file
                    let path_line: ~[&str]= req_str.split(' ').collect();
                    let file_path_str = path_line[1].slice_from(1);
                    let mut file_path = cwd.clone();

                    file_path.push(file_path_str);

                    if !file_path.exists(){
                        not_found(stream);
                        return;
                    }

                    if !cwd.is_ancestor_of(&file_path){
                        forbidden(stream);
                        return;
                    }

                    if file_path.is_dir() {
                        dir(stream,&file_path)
                    } else if file_path.is_file() {
                        serv(stream,&file_path);
                    }
                },
                None=>()
            }
            
        })
    }
}
