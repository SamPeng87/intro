extern crate env_logger;

use format;

struct FormatTests {
    layouts: Vec<&'static str>
}

impl FormatTests {
    fn new() -> FormatTests {
        FormatTests {
            layouts: vec![
                "%{string}",
                "%{string:layout} ",
                "%{string:layout}  %{tmp:abc}",
                "abcdefg%{file}:%{line} aaaa %{tmp:abc} aaa",
            ]
        }
    }
}

impl IntoIterator for FormatTests {
    type Item = &'static str;
    type IntoIter = ::std::vec::IntoIter<&'static str>;

    fn into_iter(self) -> Self::IntoIter {
        self.layouts.into_iter()
    }
}
