use crate::{Error, Result};
use openidconnect::{HttpRequest, HttpResponse};
use std::time::Duration;
use url::Url;

#[derive(Clone)]
pub(crate) struct BoundedHttp {
    client: reqwest::Client,
    origin: Url,
}
impl BoundedHttp {
    pub(crate) fn new(origin: Url) -> Result<Self> {
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(1)
            .build()
            .map_err(|_| Error::Http)?;
        Ok(Self { client, origin })
    }
    pub(crate) fn validate(&self, url: &Url) -> Result<()> {
        if url.origin() != self.origin.origin()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(Error::Configuration);
        }
        Ok(())
    }
    pub(crate) async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let (parts, body) = request.into_parts();
        if body.len() > 16384 {
            return Err(Error::Http);
        }
        let url = Url::parse(&parts.uri.to_string()).map_err(|_| Error::Http)?;
        self.validate(&url)?;
        let mut response = self
            .client
            .request(parts.method, url)
            .headers(parts.headers)
            .body(body)
            .send()
            .await
            .map_err(|_| Error::Http)?;
        if response.content_length().is_some_and(|n| n > 262144) {
            return Err(Error::Http);
        }
        let mut builder = http::Response::builder().status(response.status());
        *builder.headers_mut().ok_or(Error::Http)? = response.headers().clone();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| Error::Http)? {
            if bytes.len() + chunk.len() > 262144 {
                return Err(Error::Http);
            }
            bytes.extend_from_slice(&chunk);
        }
        builder.body(bytes).map_err(|_| Error::Http)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        thread,
    };
    fn serve_once(
        handler: impl FnOnce(TcpStream) + Send + 'static,
    ) -> (Url, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        listener.set_nonblocking(true).unwrap();
        let task = thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(std::time::Instant::now() < deadline);
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(error) => panic!("Owned fixture accept failed: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut headers = Vec::new();
            let mut byte = [0];
            while !headers.ends_with(b"\r\n\r\n") {
                assert!(headers.len() < 8192);
                stream.read_exact(&mut byte).unwrap();
                headers.push(byte[0]);
            }
            handler(stream);
        });
        (url, task)
    }
    fn server(response: Vec<u8>) -> (Url, thread::JoinHandle<()>) {
        serve_once(move |mut stream| {
            // Early rejection may close the oversized response socket.
            let _ = stream.write_all(&response);
        })
    }
    #[tokio::test]
    async fn stalled_provider_requests_support_cancellation_and_total_deadline() {
        for cancel in [true, false] {
            let (stop, stopped) = std::sync::mpsc::channel();
            let (ready, started) = tokio::sync::oneshot::channel();
            let (url, server) = serve_once(move |_stream| {
                let _ = ready.send(());
                let _ = stopped.recv_timeout(Duration::from_secs(7));
            });
            let http = BoundedHttp::new(url.clone()).unwrap();
            let request = http::Request::get(url.as_str()).body(Vec::new()).unwrap();
            let now = std::time::Instant::now();
            let task = tokio::spawn(async move { http.send(request).await });
            tokio::time::timeout(Duration::from_secs(2), started)
                .await
                .unwrap()
                .unwrap();
            if cancel {
                task.abort();
                let stopped = tokio::time::timeout(Duration::from_secs(1), task)
                    .await
                    .unwrap()
                    .unwrap_err();
                assert!(stopped.is_cancelled());
            } else {
                assert!(matches!(task.await.unwrap(), Err(Error::Http)));
                assert!(now.elapsed() >= Duration::from_secs(4));
                assert!(now.elapsed() < Duration::from_secs(8));
            }
            let _ = stop.send(());
            server.join().unwrap();
        }
    }
    #[tokio::test]
    async fn response_size_limits_cover_both_advertised_and_streamed_bodies() {
        let mut chunked =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n40001\r\n"
                .to_vec();
        chunked.extend(vec![b'x'; 262145]);
        chunked.extend_from_slice(b"\r\n0\r\n\r\n");
        for response in [
            b"HTTP/1.1 200 OK\r\nContent-Length: 262145\r\nConnection: close\r\n\r\n".to_vec(),
            chunked,
        ] {
            let (url, task) = server(response);
            let http = BoundedHttp::new(url.clone()).unwrap();
            let request = http::Request::get(url.as_str()).body(Vec::new()).unwrap();
            assert!(matches!(http.send(request).await, Err(Error::Http)));
            task.join().unwrap();
        }
    }
    #[tokio::test]
    async fn redirects_are_returned_without_following_and_foreign_origins_fail_before_io() {
        let (url,task)=server(b"HTTP/1.1 302 Found\r\nLocation: http://203.0.113.1/never-contact\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec());
        let http = BoundedHttp::new(url.clone()).unwrap();
        let response = http
            .send(http::Request::get(url.as_str()).body(Vec::new()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 302);
        task.join().unwrap();
        for remote in [
            "http://203.0.113.1/",
            "http://127.0.0.1:1/",
            "https://127.0.0.1:1/",
        ] {
            assert!(matches!(
                http.send(http::Request::get(remote).body(Vec::new()).unwrap())
                    .await,
                Err(Error::Configuration)
            ));
        }
    }
}
