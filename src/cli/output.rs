use std::io::Write;

pub struct Output<'a> {
    pub handle: Box<&'a mut dyn Write>,
}

impl Output<'_> {
    pub fn new(handle: &mut dyn Write) -> Output {
        Output {
            handle: Box::new(handle),
        }
    }
}
