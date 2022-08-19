pub mod bio;
pub mod callbacks;
pub mod error;
#[cfg(feature = "pkey")]
pub mod pkey;
#[cfg(feature = "rsa")]
pub mod rsa;
pub mod ssl;
pub mod util;
pub mod version;
pub mod x509;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TLSVersion {
    V1_3,
    V1_2,
}


#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use crate::ssl::{Ssl, SslContext, SslMethod, SslStream, SslVerifyMode};

    #[test]
    fn test() {
        let mut client_ctx = SslContext::new(SslMethod::tls_client_13()).unwrap();

        client_ctx.set_verify(SslVerifyMode::NONE);

        let client = Ssl::new(&client_ctx).unwrap();


        println!("Starting");

        let google = TcpStream::connect("google.com:443").unwrap();
        let mut client_stream = SslStream::new(client, google).unwrap();

        println!("Writing");

        client_stream.write("GET /\n".as_bytes()).unwrap();
        client_stream.flush().unwrap();

        println!("Reading");
        let mut out = [0; 128];
        client_stream.read(&mut out).unwrap();

        println!("{}", std::str::from_utf8(&out).unwrap());

    }
}
