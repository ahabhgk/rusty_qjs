pub trait Extension {
    // fn extend()
}

pub struct ContextBuilder {
    extensions: Vec<Box<dyn Extension>>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    // pub fn extend(self, )
}
