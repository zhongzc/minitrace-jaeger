mod errors;
mod reporter;
mod thrift;

pub use errors::Result;
pub use reporter::*;

#[cfg(test)]
mod tests {
    #[derive(Debug)]
    #[repr(u32)]
    enum Event {
        Parent,
        Child,
    }

    #[test]
    fn it_works() {
        let (tx, mut rx) = minitrace::Collector::bounded(5);
        {
            let span = minitrace::new_span_root(tx, Event::Parent as u32);
            let _g = span.enter();
            std::thread::sleep(std::time::Duration::from_millis(10));
            {
                let span = minitrace::new_span(Event::Child as u32);
                let _g = span.enter();
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let spans = rx.collect().unwrap();
        let reporter = crate::JaegerCompactReporter::new("minitrace_demo").unwrap();
        reporter
            .report(&spans, |tag| {
                format!("{:?}", unsafe { std::mem::transmute::<_, Event>(tag) })
            })
            .unwrap();
    }
}
