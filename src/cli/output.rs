use std::io::Write;

pub struct Output<'a> {
    pub handle: &'a mut dyn Write,
}

impl Output<'_> {
    pub fn new(handle: &mut dyn Write) -> Output {
        Output {
            handle,
        }
    }
}
