use super::*;

pub(super) struct CountingStream<E, BE> {
    pub(super) items: VecDeque<Result<E, BE>>,
    pub(super) consumed: std::sync::Arc<AtomicUsize>,
}

impl<E, BE> Unpin for CountingStream<E, BE> {}

impl<E, BE> Stream for CountingStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let next = this.items.pop_front();
        if next.is_some() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
        }
        Poll::Ready(next)
    }
}

pub(super) struct GatedCountingStream<E, BE> {
    pub(super) first: Option<Result<E, BE>>,
    pub(super) rest: VecDeque<Result<E, BE>>,
    pub(super) gate_rx: Option<oneshot::Receiver<()>>,
    pub(super) consumed: std::sync::Arc<AtomicUsize>,
}

impl<E, BE> Unpin for GatedCountingStream<E, BE> {}

impl<E, BE> Stream for GatedCountingStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(first) = this.first.take() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
            return Poll::Ready(Some(first));
        }

        if let Some(gate_rx) = &mut this.gate_rx {
            match Pin::new(gate_rx).poll(cx) {
                Poll::Ready(_) => {
                    this.gate_rx = None;
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }

        let next = this.rest.pop_front();
        if next.is_some() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
        }
        Poll::Ready(next)
    }
}

pub(super) struct ControlledEndStream<E, BE> {
    pub(super) first: Option<Result<E, BE>>,
    pub(super) finish_rx: oneshot::Receiver<()>,
}

impl<E, BE> Unpin for ControlledEndStream<E, BE> {}

impl<E, BE> Stream for ControlledEndStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(first) = this.first.take() {
            return Poll::Ready(Some(first));
        }

        match Pin::new(&mut this.finish_rx).poll(cx) {
            Poll::Ready(_) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
