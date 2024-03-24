pub mod position;
pub mod concrete;

pub trait Ticker<S>: Fn(&mut S) {}
impl<T, S> Ticker<S> for T where T: Fn(&mut S) {}

pub trait Tickable {
    fn tick(&mut self);
}

pub struct TickComponent<A> {
    tickers: Vec<Box<dyn Ticker<A>>>,
}
impl <A>TickComponent<A> {
    pub fn append_ticker(&mut self, ticker: Box<dyn Ticker<A>>) {
        self.tickers.push(ticker);
    }
}
