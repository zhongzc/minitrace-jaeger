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
        let (root, collector) = minitrace::trace_enable(Event::Parent as u32);
        {
            let _guard = root;
            std::thread::sleep(std::time::Duration::from_millis(10));
            {
                let _guard = minitrace::new_span(Event::Child as u32);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let spans = collector.collect();
        let reporter = crate::JaegerCompactReporter::new("minitrace_demo").unwrap();
        reporter
            .report(&spans, |tag| {
                format!("{:?}", unsafe { std::mem::transmute::<_, Event>(tag) })
            })
            .unwrap();
    }
}
