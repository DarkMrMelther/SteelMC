use steel_utils::logger::RequestKind;
use tokio::sync::oneshot;

pub struct InputRequest {
    pub sender: oneshot::Sender<String>,
    options: RequestKind,
}

impl InputRequest {
    pub fn new(options: RequestKind) -> (Self, oneshot::Receiver<String>) {
        let (sender, receiver) = oneshot::channel();
        (Self { sender, options }, receiver)
    }

    pub fn options(&self, input: &str) -> Vec<String> {
        let options = match self.options {
            RequestKind::String => vec![],
            RequestKind::Options(options) => options.iter().map(|o| o.to_string()).collect(),
            RequestKind::Numeral(range) => {
                range.clone().into_iter().map(|n| n.to_string()).collect()
            }
            RequestKind::Bool => vec!["yes".into(), "no".into()],
        };
        options
            .into_iter()
            .filter(|opt| opt.trim().starts_with(input))
            .collect()
    }
}
