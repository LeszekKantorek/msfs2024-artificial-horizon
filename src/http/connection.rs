//! Socket deadlines enforce resource release even when HTTP cannot poll its body.
use axum::serve::Listener;
use std::{
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::watch,
    time::{Duration, Instant, Sleep, sleep_until},
};

const WRITE_STALL: Duration = Duration::from_secs(30);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
type ShutdownDeadline = Pin<Box<dyn Future<Output = ()> + Send>>;

pub(super) struct DeadlineListener {
    listener: TcpListener,
    closing: watch::Receiver<Option<Instant>>,
}
impl DeadlineListener {
    pub(super) fn new(listener: TcpListener, closing: watch::Receiver<Option<Instant>>) -> Self {
        Self { listener, closing }
    }
}
impl Listener for DeadlineListener {
    type Io = DeadlineIo<TcpStream>;
    type Addr = SocketAddr;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        // Retain Axum's accept-error logging/backoff rather than spin on failures.
        let (socket, address) = Listener::accept(&mut self.listener).await;
        (DeadlineIo::new(socket, self.closing.clone()), address)
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

pub(super) struct DeadlineIo<T> {
    inner: T,
    shutdown: ShutdownDeadline,
    shutdown_expired: bool,
    stalled: Option<Pin<Box<Sleep>>>,
}
impl<T> DeadlineIo<T> {
    fn new(inner: T, mut closing: watch::Receiver<Option<Instant>>) -> Self {
        let shutdown = Box::pin(async move {
            loop {
                let started = *closing.borrow_and_update();
                if let Some(started) = started {
                    sleep_until(started + SHUTDOWN_GRACE).await;
                    return;
                }
                if closing.changed().await.is_err() {
                    return;
                }
            }
        });
        Self {
            inner,
            shutdown,
            shutdown_expired: false,
            stalled: None,
        }
    }
    fn check_shutdown(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
        if self.shutdown_expired || self.shutdown.as_mut().poll(cx).is_ready() {
            self.shutdown_expired = true;
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "HTTP shutdown grace expired",
            ))
        } else {
            Ok(())
        }
    }
    fn write_result<R>(
        &mut self,
        cx: &mut Context<'_>,
        result: Poll<io::Result<R>>,
    ) -> Poll<io::Result<R>> {
        if result.is_ready() {
            self.stalled = None;
            return result;
        }
        let deadline = self
            .stalled
            .get_or_insert_with(|| Box::pin(sleep_until(Instant::now() + WRITE_STALL)));
        if deadline.as_mut().poll(cx).is_ready() {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "HTTP write made no progress for 30 seconds",
            )))
        } else {
            Poll::Pending
        }
    }
}
impl<T: AsyncRead + Unpin> AsyncRead for DeadlineIo<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        this.check_shutdown(cx)?;
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}
impl<T: AsyncWrite + Unpin> AsyncWrite for DeadlineIo<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        this.check_shutdown(cx)?;
        let result = Pin::new(&mut this.inner).poll_write(cx, buf);
        this.write_result(cx, result)
    }
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        this.check_shutdown(cx)?;
        let result = Pin::new(&mut this.inner).poll_write_vectored(cx, bufs);
        this.write_result(cx, result)
    }
    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        this.check_shutdown(cx)?;
        let result = Pin::new(&mut this.inner).poll_flush(cx);
        this.write_result(cx, result)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        this.check_shutdown(cx)?;
        let result = Pin::new(&mut this.inner).poll_shutdown(cx);
        this.write_result(cx, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt, duplex},
        time::advance,
    };

    // A transport may accept bytes but never finish flushing/shutting down.
    struct PendingFlush;
    impl AsyncWrite for PendingFlush {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            bytes: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Ok(bytes.len()))
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Pending
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Pending
        }
    }

    #[tokio::test(start_paused = true)]
    async fn pending_flush_times_out_and_shutdown_also_has_a_deadline() {
        let (closing, closed) = watch::channel(None);
        let mut socket = DeadlineIo::new(PendingFlush, closed.clone());
        socket.write_all(b"data").await.unwrap();
        let mut flush = Box::pin(socket.flush());
        assert!(flush.as_mut().now_or_never().is_none());
        advance(WRITE_STALL).await;
        assert_eq!(flush.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
        // A timed-out connection is terminal; shutdown uses a separate connection.
        let mut socket = DeadlineIo::new(PendingFlush, closed);
        closing.send_replace(Some(Instant::now()));
        let mut shutdown = Box::pin(socket.shutdown());
        assert!(shutdown.as_mut().now_or_never().is_none());
        advance(SHUTDOWN_GRACE).await;
        assert_eq!(shutdown.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test(start_paused = true)]
    async fn blocked_write_times_out_after_thirty_seconds() {
        let (_closing, closed) = watch::channel(None);
        let (socket, _peer) = duplex(1);
        let mut socket = DeadlineIo::new(socket, closed);
        socket.write_all(b"x").await.unwrap();
        let mut write = Box::pin(socket.write_all(b"y"));
        assert!(write.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(29)).await;
        assert!(write.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(1)).await;
        assert_eq!(write.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test(start_paused = true)]
    async fn successful_write_renews_deadline_and_idle_time_is_not_a_stall() {
        let (_closing, closed) = watch::channel(None);
        let (socket, mut peer) = duplex(1);
        let mut socket = DeadlineIo::new(socket, closed);
        advance(Duration::from_secs(60)).await;
        socket.write_all(b"x").await.unwrap();
        let mut write = Box::pin(socket.write_all(b"y"));
        assert!(write.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(29)).await;
        let mut byte = [0];
        peer.read_exact(&mut byte).await.unwrap();
        write.await.unwrap();
        let buffers = [io::IoSlice::new(b"z")];
        let mut vectored = Box::pin(socket.write_vectored(&buffers));
        assert!(vectored.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(29)).await;
        assert!(vectored.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(1)).await;
        assert_eq!(vectored.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    #[tokio::test(start_paused = true)]
    async fn shutdown_interrupts_blocked_read_and_uses_absolute_deadline() {
        let (closing, closed) = watch::channel(None);
        let (socket, _peer) = duplex(1);
        let mut socket = DeadlineIo::new(socket, closed);
        closing.send_replace(Some(Instant::now()));
        advance(Duration::from_secs(4)).await;
        let mut byte = [0];
        let mut read = Box::pin(socket.read_exact(&mut byte));
        assert!(read.as_mut().now_or_never().is_none());
        advance(Duration::from_secs(1)).await;
        assert_eq!(read.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            socket.read_exact(&mut byte).await.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
    }

    #[tokio::test(start_paused = true)]
    async fn shutdown_interrupts_stalled_writer_before_write_timeout() {
        let (closing, closed) = watch::channel(None);
        let (socket, _peer) = duplex(1);
        let mut socket = DeadlineIo::new(socket, closed);
        socket.write_all(b"x").await.unwrap();
        let mut write = Box::pin(socket.write_all(b"y"));
        assert!(write.as_mut().now_or_never().is_none());
        closing.send_replace(Some(Instant::now()));
        advance(SHUTDOWN_GRACE).await;
        assert_eq!(write.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
}
